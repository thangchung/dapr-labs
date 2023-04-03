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


```bash
cd build/nomad/jobs
nomad job run product-api-dapr.nomad.hcl
nomad job stop product-api-dapr
nomad system gc
```

```bash
nomad job run traefik.nomad.hcl
nomad job run postgresdb.nomad.hcl
nomad job run counter-api-dapr.nomad.hcl
nomad job stop counter-api-dapr
nomad system gc
```
