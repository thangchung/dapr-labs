# Part 4: WebAssembly, Docker container, Dapr, and Kubernetes better together - Package the daprized coffee backend services to Docker container and deploy to Kubernetes

In this final part, we will install `k3d`, and `containerd-wasm-shims` which run `runwasi` inside. It allows us to run WASM/WASI workload with the annotation `runtimeClassName: wasmtime-spin`, and run a Docker container (containerd format). If you don't declare the previous annotation on the YAML deployment script.

Let's get started with the setup as the image below.

![docker and k8s](img/wasm-dapr-4.png)

## Setup Docker (buildx), k3d

```sh
# ref: https://github.com/docker/docker-install

> curl -fsSL https://get.docker.com -o get-docker.sh
> sh get-docker.sh --version 24.0
> docker --version
Docker version 24.0.5, build ced0996
```

```sh
# install k3d

> wget -q -O - https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash
```

## Dockerized apps

```Dockerfile
# Product API
FROM --platform=${BUILDPLATFORM} rust:1.67 AS build
RUN rustup target add wasm32-wasi
COPY . /product
WORKDIR /product
RUN cargo build --target wasm32-wasi --release

FROM scratch
COPY --from=build /product/target/wasm32-wasi/release/product_api.wasm /target/wasm32-wasi/release/product_api.wasm
COPY ./spin.toml /spin.toml
```

```sh
# Counter API, .NET 8
> dotnet publish ./counter-api/counter-api.csproj --os linux --arch x64 /t:PublishContainer -c Release
docker tag counter-api:latest ghcr.io/thangchung/dapr-labs/counter-api-polyglot:1.0.0
```

```Dockerfile
# Barista API
FROM --platform=${BUILDPLATFORM} rust:1.67 AS build
RUN rustup target add wasm32-wasi
COPY . /barista
WORKDIR /barista
RUN cargo build --target wasm32-wasi --release

FROM scratch
COPY --from=build /barista/target/wasm32-wasi/release/barista_api.wasm /target/wasm32-wasi/release/barista_api.wasm
COPY ./spin.toml /spin.toml
```

```Dockerfile
# Kitchen API
FROM --platform=${BUILDPLATFORM} rust:1.67 AS build
RUN rustup target add wasm32-wasi
COPY . /kitchen
WORKDIR /kitchen
RUN cargo build --target wasm32-wasi --release

FROM scratch
COPY --from=build /kitchen/target/wasm32-wasi/release/kitchen_api.wasm /target/wasm32-wasi/release/kitchen_api.wasm
COPY ./spin.toml /spin.toml
ENTRYPOINT ["/"]
```

Build and push it into `GitHub artifacts`:

```sh
> docker login ghcr.io -u <your username>
```

It asks you to provide the password (PAT), please go to your developer profile to generate it.

```sh
> docker buildx build -f Dockerfile --platform wasi/wasm,linux/amd64,linux/arm64 -t ghcr.io/thangchung/dapr-labs/product-api-spin:1.0.0 . --push

> docker push ghcr.io/thangchung/dapr-labs/counter-api-polyglot:1.0.0

> docker buildx build -f Dockerfile --platform wasi/wasm,linux/amd64,linux/arm64 -t ghcr.io/thangchung/dapr-labs/barista-api-spin:1.0.0 . --push

> docker buildx build -f Dockerfile --platform wasi/wasm,linux/amd64,linux/arm64 -t ghcr.io/thangchung/dapr-labs/kitchen-api-spin:1.0.0 . --push
```

## Ship them to Kubernetes

```yaml
# Product API
apiVersion: apps/v1
kind: Deployment
metadata:
  name: product-api
spec:
  replicas: 1
  selector:
    matchLabels:
      app: product-api
  template:
    metadata:
      labels:
        app: product-api
      annotations:
        dapr.io/enabled: "true"
        dapr.io/app-id: "product-api"
        dapr.io/app-port: "80"
        dapr.io/enable-api-logging: "true"
    spec:
      runtimeClassName: wasmtime-spin
      containers:
        - name: product-api
          image: ghcr.io/thangchung/dapr-labs/product-api-spin:1.0.1
          command: ["/"]
          env:
            - name: RUST_BACKTRACE
              value: "1"
          resources: # limit the resources to 128Mi of memory and 100m of CPU
            limits:
              cpu: 100m
              memory: 128Mi
            requests:
              cpu: 100m
              memory: 128Mi
---
apiVersion: v1
kind: Service
metadata:
  name: product-api
spec:
  type: LoadBalancer
  ports:
    - protocol: TCP
      port: 5001
      targetPort: 80
  selector:
    app: product-api
```

```yaml
# Counter API
apiVersion: apps/v1
kind: Deployment
metadata:
  name: counter-api
spec:
  replicas: 1
  selector:
    matchLabels:
      app: counter-api
  template:
    metadata:
      labels:
        app: counter-api
      annotations:
        dapr.io/enabled: "true"
        dapr.io/app-id: "counter-api"
        dapr.io/app-port: "8080"
        dapr.io/enable-api-logging: "true"
    spec:
      containers:
        - name: counter-api
          image: ghcr.io/thangchung/dapr-labs/counter-api-polyglot:1.0.0
          env:
            - name: ProductCatalogAppDaprName
              value: "product-api"
          resources: # limit the resources to 128Mi of memory and 100m of CPU
            limits:
              cpu: 100m
              memory: 128Mi
            requests:
              cpu: 100m
              memory: 128Mi
---
apiVersion: v1
kind: Service
metadata:
  name: counter-api
spec:
  type: LoadBalancer
  ports:
    - protocol: TCP
      port: 5002
      targetPort: 8080
  selector:
    app: counter-api
```

```yaml
# Barista API
apiVersion: apps/v1
kind: Deployment
metadata:
  name: barista-api
spec:
  replicas: 1
  selector:
    matchLabels:
      app: barista-api
  template:
    metadata:
      labels:
        app: barista-api
      annotations:
        dapr.io/enabled: "true"
        dapr.io/app-id: "barista-api"
        dapr.io/app-port: "80"
        dapr.io/enable-api-logging: "true"
    spec:
      runtimeClassName: wasmtime-spin
      containers:
        - name: barista-api
          image: ghcr.io/thangchung/dapr-labs/barista-api-spin:1.0.0
          # imagePullPolicy: Always
          command: ["/"]
          env:
            - name: RUST_BACKTRACE
              value: "1"
            - name: DAPR_URL
              value: "http://localhost:3500"
          resources: # limit the resources to 128Mi of memory and 100m of CPU
            limits:
              cpu: 100m
              memory: 128Mi
            requests:
              cpu: 100m
              memory: 128Mi
---
apiVersion: v1
kind: Service
metadata:
  name: barista-api
spec:
  type: LoadBalancer
  ports:
    - protocol: TCP
      port: 5003
      targetPort: 80
  selector:
    app: barista-api
```

```yaml
# Kitchen API
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kitchen-api
spec:
  replicas: 1
  selector:
    matchLabels:
      app: kitchen-api
  template:
    metadata:
      labels:
        app: kitchen-api
      annotations:
        dapr.io/enabled: "true"
        dapr.io/app-id: "kitchen-api"
        dapr.io/app-port: "80"
        dapr.io/enable-api-logging: "true"
    spec:
      runtimeClassName: wasmtime-spin
      containers:
        - name: kitchen-api
          image: ghcr.io/thangchung/dapr-labs/kitchen-api-spin:1.0.0
          # imagePullPolicy: Always
          command: ["/"]
          env:
            - name: RUST_BACKTRACE
              value: "1"
            - name: DAPR_URL
              value: "http://localhost:3500"
          resources: # limit the resources to 128Mi of memory and 100m of CPU
            limits:
              cpu: 100m
              memory: 128Mi
            requests:
              cpu: 100m
              memory: 128Mi
---
apiVersion: v1
kind: Service
metadata:
  name: kitchen-api
spec:
  type: LoadBalancer
  ports:
    - protocol: TCP
      port: 5004
      targetPort: 80
  selector:
    app: kitchen-api
```

```yaml
# ingress.yaml
# Middleware
# Strip prefix /spin
apiVersion: traefik.containo.us/v1alpha1
kind: Middleware
metadata:
  name: strip-prefix
spec:
  stripPrefix:
    forceSlash: false
    prefixes:
      - /p
      - /c
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: polyglot-wasm-ingress
  annotations:
    ingress.kubernetes.io/ssl-redirect: "false"
    kubernetes.io/ingress.class: traefik
    traefik.ingress.kubernetes.io/router.middlewares: default-strip-prefix@kubernetescrd
spec:
  rules:
    - http:
        paths:
          - path: /p
            pathType: Prefix
            backend:
              service:
                name: product-api
                port:
                  number: 5001
          - path: /c
            pathType: Prefix
            backend:
              service:
                name: counter-api
                port:
                  number: 5002
```

```sh
# create Kubernetes cluster for WASM/WASI
> k3d cluster create wasm-cluster --image ghcr.io/deislabs/containerd-wasm-shims/examples/k3d:v0.9.0 -p "8081:80@loadbalancer" --agents 2
```

```sh
# add redis
> helm install my-redis oci://registry-1.docker.io/bitnamicharts/redis --set architecture=standalone --set global.redis.password=P@ssw0rd
```

```sh
# for demo only, otherwise need use Dapr with Helm chart
> dapr init -k --runtime-version 1.11.2
```

```sh
# remember to edit dapr component YAML, and put password for redis
> kubectl apply -f components-k8s

> kubectl apply -f iac/kind-spin
```

Make sure you can query:

```sh
> kubectl get component
NAME            AGE
baristapubsub   2d1h
kitchenpubsub   2d1h
statestore      2d1h

> kubectl get subscription
NAME                           AGE
barista-ordered-subscription   2d1h
barista-updated-subscription   2d1h
kitchen-ordered-subscription   2d1h
kitchen-updated-subscription   2d1h

> kubectl get po
NAME                           READY   STATUS    RESTARTS       AGE
my-redis-master-0              1/1     Running   24 (55m ago)   10d
counter-api-8bdc488b7-h96pw    2/2     Running   16 (55m ago)   2d1h
product-api-8ccbc56b-ql5bt     2/2     Running   12 (54m ago)   2d1h
kitchen-api-54f7c588cb-ffnzl   2/2     Running   12 (55m ago)   2d1h
barista-api-6f5b4d6fb8-pxbqc   2/2     Running   12 (55m ago)   2d1h

> kubectl get svc
NAME                TYPE           CLUSTER-IP      EXTERNAL-IP                        PORT(S)
              AGE
kubernetes          ClusterIP      10.43.0.1       <none>                             443/TCP
              10d
my-redis-headless   ClusterIP      None            <none>                             6379/TCP
              10d
my-redis-master     ClusterIP      10.43.109.123   <none>                             6379/TCP
              10d
barista-api-dapr    ClusterIP      None            <none>                             80/TCP,50001/TCP,50002/TCP,9090/TCP   2d1h
counter-api-dapr    ClusterIP      None            <none>                             80/TCP,50001/TCP,50002/TCP,9090/TCP   2d1h
kitchen-api-dapr    ClusterIP      None            <none>                             80/TCP,50001/TCP,50002/TCP,9090/TCP   2d1h
product-api-dapr    ClusterIP      None            <none>                             80/TCP,50001/TCP,50002/TCP,9090/TCP   2d1h
product-api         LoadBalancer   10.43.58.9      172.19.0.2,172.19.0.4,172.19.0.5   5001:30896/TCP
              2d1h
kitchen-api         LoadBalancer   10.43.105.225   172.19.0.2,172.19.0.4,172.19.0.5   5004:32611/TCP
              2d1h
counter-api         LoadBalancer   10.43.19.86     172.19.0.2,172.19.0.4,172.19.0.5   5002:30630/TCP
              2d1h
barista-api         LoadBalancer   10.43.105.93    172.19.0.2,172.19.0.4,172.19.0.5   5003:31258/TCP
              2d1h

> kubectl get ing
NAME                    CLASS    HOSTS   ADDRESS                            PORTS   AGE
polyglot-wasm-ingress   <none>   *       172.19.0.2,172.19.0.4,172.19.0.5   80      2d1h
```

If everything is okay, then we can play around with Rest Client at [client.k3d.http](https://github.com/thangchung/dapr-labs/blob/main/polyglot/client.k3d.http).

## Summary

So we have already walked through 4 parts of how can we build the polyglot apps with Docker, WebAssembly (Spin), Dapr, and Kubernetes (k3d). There are still obstacles, but the future is bright for WASM apps on Kubernetes due to the very active process from the community.

What's next? The WebAssembly/WASI has a [clear roadmap](https://bytecodealliance.org/articles/webassembly-the-updated-roadmap-for-developers). In WasmCon 2023 recently, they mentioned [`Component Model`](https://component-model.bytecodealliance.org/) and I think that is [a final abstraction](https://cosmonic.com/blog/industry/webassembly-components-the-final-abstraction) in the computing unit. It helps a lot in building what [Luke Wagner](https://github.com/lukewagner) calls [`Modularity without microservices`](https://wasmcon2023.sched.com/event/1P96K/keynote-what-is-a-component-and-why-luke-wagner-distinguished-engineer-fastly?iframe=no&w=100%&sidebar=yes&bg=no), and we will invest time on it as well. See you in the next posts to dive more into this kind of component model.

Stay stun!
