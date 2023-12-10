# Get starting

## Product APIs

Create new APIs with Rust

```bash
> spin new http-rust product-api --accept-defaults && cd product-api
```

```bash
dapr run \
    --app-id productapi \
    --app-port 5001 \
    --resources-path ../components \
    --config ../components/daprConfig.yaml \
    -- spin up --listen 0.0.0.0:5001
```

Run APIs

```bash
> cd product-api
> spin build
> spin up
```

## Refs

- https://github.com/ThorstenHans/spin_book_api