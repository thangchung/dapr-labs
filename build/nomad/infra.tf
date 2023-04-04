provider "nomad" {
  address = "http://localhost:4646"
  version = "~> 1.4"
}

resource "nomad_job" "traefik" {
  jobspec = file("${path.module}/jobs/traefik.nomad.hcl")
}

resource "nomad_job" "postgres_db" {
  jobspec    = file("${path.module}/jobs/postgresdb.nomad.hcl")
  depends_on = [nomad_job.traefik]
}

# resource "nomad_job" "rabbitmq" {
#   jobspec    = file("${path.module}/jobs/rabbitmq.nomad.hcl")
#   depends_on = [nomad_job.traefik]
# }

resource "nomad_job" "redis" {
  jobspec    = file("${path.module}/jobs/redis.nomad.hcl")
  depends_on = [nomad_job.traefik]
}
