# CoffeeShop Polyglot

![coffeeshop-polyglot-highlevelarchirecture](assets/coffeeshop-polyglot-highlevelarchirecture.png)

## Get starting

See how to init [Dapr](#dapr) as below

```sh
docker compose up
```

```sh
cd product-api
dapr run \
    --app-id productapi \
    --app-port 5001 \
    --resources-path ../components \
    --config ../components/daprConfig.yaml \
    -- spin up --listen 0.0.0.0:5001
```

```sh
dapr list # see if it runs or not?
```

```sh
dapr run \
    --app-id counterapi \
    --app-port 5002 \
    --resources-path components \
    --config components/daprConfig.yaml \
    -- dotnet run --project counter-api/CounterApi.csproj
```

Then, playing around with it at [client.local.http](client.local.http)!

## Install dotnet on Ubuntu 22.04

```sh
sudo ./dotnet-install.sh -v 8.0.100-preview.4.23260.5 --install-dir /usr/share/dotnet
```

## Dapr

Upgrade dapr CLI

```sh
wget -q https://raw.githubusercontent.com/dapr/cli/master/install/install.sh -O - | /bin/bash -s 1.11.0-rc.2
```

Init dapr 1.11-rc

```sh
dapr init --runtime-version 1.11.0-rc.7
```
