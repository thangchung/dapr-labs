# CoffeeShop Polyglot on Dapr

![coffeeshop-polyglot-highlevelarchirecture](assets/coffeeshop-polyglot-highlevelarchirecture.png)

## Get starting

See how to init [Dapr](#dapr) as below

```sh
docker compose up
```

```sh
make run-product-dapr
```

```sh
make run-counter-dapr
```

```sh
make run-barista-dapr
```

```sh
make run-kitchen-dapr
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

## Kubernetes

### kwasm (kind)

TODO

### AKS

TODO
