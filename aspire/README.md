# Get starting

## Push wasm directly to registry

```sh
spin registry push ttl.sh/thangchung-test-spin:1h --build
```

```sh
dotnet publish ./WebApp/WebApp.csproj --os linux --arch x64 /t:PublishContainer -c Release
```

## Troubleshooting

- After restarted AKS cluster, then our apps could not be started

=> run

```sh
kubectl annotate node --all kwasm.sh/kwasm-node=true
```

Reported Issue: https://github.com/spinkube/azure/issues/24

- Cannot set `imagePullPolicy: Always` in SpinApp manifest => edits deployment and changes to `Always`.

## Refs

- [distributed-todo-app](https://github.com/fermyon/enterprise-architectures-and-patterns/tree/main/distributed-todo-app)
  - [Simple log](https://www.fermyon.com/blog/exploring_variables#implementing-our-application)
- [coffeeshop-polyglot](https://github.com/thangchung/dapr-labs/tree/main/polyglot/product-api)
- [Aspire.Spin](https://github.com/fermyon/Aspire.Spin/tree/main/Aspire.Hosting.Spin)
- Dapr references
  - https://github.com/spinkube/containerd-shim-spin/tree/b60027b0d3c050e028107736344ccc589e0ab31b/images/spin-dapr
  - https://github.com/dotnet/aspire/blob/main/playground/dapr/Dapr.AppHost/Program.cs
  - https://github.com/fvandillen/futuretech-dapr-aspire/blob/main/Futuretech/Futuretech.AppHost/Program.cs
  - https://github.com/fvandillen/dapr-aspire/blob/main/DaprAspire/DaprAspire.AppHost/Program.cs
- https://anthonysimmon.com/referencing-external-docker-containers-dotnet-aspire-custom-resources/
