# Starting app with dapr

```bash
dapr init
```

```bash
dapr run --app-port 5001 --app-id product-api --dapr-http-port 3500 --resources-path ../../build/dapr/components --config ../../build/dapr/components/daprConfig.yaml -- go run github.com/thangchung/go-coffeeshop/cmd/product
```

```bash
dapr run `
    --app-id product-api `
    --app-port 5001 `
    --dapr-http-port 3500 `
    --resources-path build/dapr/components `
    --config build/dapr/components/daprConfig.yaml `
    cd cmd/product && go mod tidy && go mod download && CGO_ENABLED=0 go run github.com/thangchung/go-coffeeshop/cmd/product
```
