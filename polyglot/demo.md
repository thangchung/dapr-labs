# CNCF meetup event November 2023

## Run on local

```sh
> make -j daprized-apps
```

Test it, [client.http](client.http)

## Install Dapr components

```sh
> kubectl apply -f ./components-k8s
> kubectl get component
```

## Install WASM/WASI services

```sh
> kubectl apply -f ./iac/kind-spin
> kubectl get po,svc
```

Play around, [client.k3d.http](client.k3d.http)

## Clean up

```sh
> kubectl delete -f ./iac/kind-spin
> kubectl delete -f ./components-k8s
> # helm delete my-redis
```
