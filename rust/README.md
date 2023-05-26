# Rust CoffeeShop App

[Rest API specs](api-specs.md)

## Env

Create .env file

```bash
HOST=0.0.0.0
DATABASE_URL=postgres://postgres:P@ssw0rd@127.0.0.1/postgres
DAPR_URL=http://localhost:42573 #your Dapr product port on local, type <dapr list> to get it
DAPR_PRODUCT_APP=productapi
```

## Dapr

```bash
dapr run \
    --app-id productapi \
    --app-port 5001 \
    --resources-path components \
    --config components/daprConfig.yaml \
    -- cargo run --bin product_api
```

```bash
dapr run \
    --app-id counterapi \
    --app-port 5002 \
    --resources-path components \
    --config components/daprConfig.yaml \
    -- cargo run --bin counter_api
```

```bash
dapr run \
    --app-id baristaapi \
    --app-port 5003 \
    --resources-path components \
    --config components/daprConfig.yaml \
    -- cargo run --bin barista_api
```

```bash
dapr run \
    --app-id kitchenapi \
    --app-port 5004 \
    --resources-path components \
    --config components/daprConfig.yaml \
    -- cargo run --bin kitchen_api
```

- https://docs.dapr.io/reference/environment/

## Database up

Before `docker compose up`, pls remember to run `sudo rm -rf postgres-data`

```sql
sea-orm-cli generate entity -l -s order -o crates/counter_entity/src
sea-orm-cli generate entity -l -s barista -o crates/barista_entity/src
sea-orm-cli generate entity -l -s kitchen -o crates/kitchen_entity/src
```

- https://gist.github.com/ynwd/f39c78fc4c62b0116425f333be2b9f77

## Ref projects

- [SeaQL/sea-orm/axum_example](https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example)
- [vietnam-devs/coolstore-microservices](https://github.com/vietnam-devs/coolstore-microservices/tree/feature/upgrade-net6/src/rust)
- [fermyon/spin](https://github.com/fermyon/spin)
- [AxumCourse/axum-with-seaorm](https://github.com/AxumCourse/axum-with-seaorm)
- [solidiquis/knodis](https://github.com/solidiquis/knodis)

## Ref articles

- https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
- https://www.thorsten-hans.com/working-with-environment-variables-in-rust/
- https://chrismcg.com/2019/04/30/deserializing-optional-datetimes-with-serde/
- https://github.com/programatik29/axum-tutorial/blob/master/tutorial/04-generate-random-number.md
