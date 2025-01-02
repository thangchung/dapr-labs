# Get starting

## Push wasm directly to registry

```sh
spin registry push ttl.sh/thangchung-test-spin:1h --build
```

```sh
dotnet publish ./WebApp/WebApp.csproj --os linux --arch x64 /t:PublishContainer -c Release
```

## Refs

- [distributed-todo-app](https://github.com/fermyon/enterprise-architectures-and-patterns/tree/main/distributed-todo-app)
- [coffeeshop-polyglot](https://github.com/thangchung/dapr-labs/tree/main/polyglot/product-api)
- [Aspire.Spin](https://github.com/fermyon/Aspire.Spin/tree/main/Aspire.Hosting.Spin)
- Dapr references
  - https://github.com/dotnet/aspire/blob/main/playground/dapr/Dapr.AppHost/Program.cs
  - https://github.com/fvandillen/futuretech-dapr-aspire/blob/main/Futuretech/Futuretech.AppHost/Program.cs
  - https://github.com/fvandillen/dapr-aspire/blob/main/DaprAspire/DaprAspire.AppHost/Program.cs
- https://anthonysimmon.com/referencing-external-docker-containers-dotnet-aspire-custom-resources/
