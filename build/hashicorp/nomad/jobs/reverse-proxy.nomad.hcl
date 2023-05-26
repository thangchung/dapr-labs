job "reverse-proxy" {
  datacenters = ["dc1"]

  group "svc" {
    network {
      mode = "bridge"

      port "http" {
        to = 8080
      }
    }

    service {
      name = "reverse-proxy-http"
      port = "${NOMAD_PORT_http}"

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "product-api-dapr-http"
              local_bind_port  = 5001
            }
            upstreams {
              destination_name = "counter-api-dapr-http"
              local_bind_port  = 5002
            }
          }
        }
      }

      tags = [
        "traefik.enable=true",
        "traefik.consulcatalog.connect=true",
        "traefik.http.routers.api.rule=PathPrefix(`/api`)",
        "traefik.http.routers.api.middlewares=api-stripprefix",
        "traefik.http.middlewares.api-stripprefix.stripprefix.prefixes=/api",
      ]
    }

    task "reverse-proxy" {
      driver = "docker"

      config {
        image = "ghcr.io/thangchung/try-nomad-dapr/reverse-proxy:0.1.0"
        ports = ["${NOMAD_PORT_http}"]
      }

      env {
        ASPNETCORE_ENVIRONMENT = "Development"
      }

      resources {
        memory = 128
      }
    }
  }
}