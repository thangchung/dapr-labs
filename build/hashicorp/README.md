# Get starting with Nomad, Consult Connect

## Start Nomad, Consul and Vault

```bash
> cd build/local
> ./start.sh
```

> Make sure you set start.sh with execute permission => `sudo chmod +x start.sh`

## Use Terraform to provisioning all services

```bash
> cd build/nomad
> terraform init
> terraform apply
```

## Clean Up

```bash
> cd build/nomad
> terraform destroy
> cd build/local
# Ctrl + C
```

Happy hacking with HashiCorp stack!!!

http://localhost:8500

Key/Value menu

dapr/daprConfig.yaml
dapr/components/consul.yaml
dapr/components/orderup_pubsub.yaml
dapr/components/barista_pubsub.yaml
dapr/components/kitchen_pubsub.yaml

Notes: host_ip normally is `172.17.0.3`

```bash
cd build/nomad/jobs
```

```bash
nomad job run traefik.nomad.hcl
nomad job run postgresdb.nomad.hcl
nomad job run redis.nomad.hcl

nomad job stop traefik
nomad job stop postgres-db
nomad job stop redis
nomad system gc
```

```bash
nomad job run product-api-dapr.nomad.hcl
nomad job stop product-api-dapr
nomad system gc
```

```bash
nomad job run counter-api-dapr.nomad.hcl
nomad job stop counter-api-dapr
nomad system gc
```

```bash
nomad job run barista-api-dapr.nomad.hcl
nomad job stop barista-api-dapr
nomad system gc
```

```bash
nomad job run kitchen-api-dapr.nomad.hcl
nomad job stop kitchen-api-dapr
nomad system gc
```

```bash
nomad job run reverse-proxy.nomad.hcl
nomad job stop reverse-proxy
nomad system gc
```
