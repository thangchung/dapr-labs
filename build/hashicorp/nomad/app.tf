resource "nomad_job" "product_api" {
  jobspec = file("${path.module}/jobs/product-api-dapr.nomad.hcl")
}

resource "nomad_job" "counter_api" {
  jobspec    = file("${path.module}/jobs/counter-api-dapr.nomad.hcl")
  depends_on = [nomad_job.product_api]
}

resource "nomad_job" "barista_api" {
  jobspec    = file("${path.module}/jobs/barista-api-dapr.nomad.hcl")
}

resource "nomad_job" "kitchen_api" {
  jobspec    = file("${path.module}/jobs/kitchen-api-dapr.nomad.hcl")
}

resource "nomad_job" "reverse_proxy" {
  jobspec    = file("${path.module}/jobs/reverse-proxy.nomad.hcl")
  depends_on = [nomad_job.product_api, nomad_job.counter_api, nomad_job.barista_api, nomad_job.kitchen_api]
}

# resource "nomad_job" "web" {
#   jobspec    = file("${path.module}/jobs/web.nomad.hcl")
#   depends_on = [nomad_job.grpc_gw]
# }
