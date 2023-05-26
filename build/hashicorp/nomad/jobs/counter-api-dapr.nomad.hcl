job "counter-api-dapr" {
  datacenters = ["dc1"]

  group "counter-api-dapr" {
    network {
      mode = "bridge"
      port "app" { to = 5002 }
      port "http" { to = 3500 }
      port "grpc" { to = 50001 }
      port "rpc" { to = 40001 }
    }

    service {
      name         = "counter-api-dapr-http"
      port         = "${NOMAD_PORT_app}"
      address_mode = "host"

      // this is really important for traefik to inject the routers below
      connect {
        sidecar_service {}
      }

      tags = [
        "dapr",
        // "traefik.enable=true",
        // "traefik.consulcatalog.connect=true",
        // "traefik.http.routers.api.rule=PathPrefix(`/counter-api`)",
        // "traefik.http.routers.api.middlewares=counter_api_stripprefix",
        // "traefik.http.middlewares.counter_api_stripprefix.stripprefix.prefixes=/counter-api",
      ]
      meta {
        DAPR_PORT = "${NOMAD_HOST_PORT_rpc}"
        APP_PORT  = "${NOMAD_PORT_app}"
      }
    }

    task "counter-api-dapr" {
      driver = "docker"

      config {
        image = "ghcr.io/thangchung/try-nomad-dapr/counterapi:0.1.0"
        ports = ["${NOMAD_PORT_app}"]
      }

      env {
        ASPNETCORE_ENVIRONMENT       = "Development"
        ConnectionStrings__counterdb = "Server=${attr.unique.network.ip-address};Port=5432;Database=postgres;User Id=postgres;Password=P@ssw0rd"
        ProductCatalogAppDaprName    = "product-api-dapr-http"
      }

      resources {
        memory = 200
      }
    }

    task "daprd" {
      driver = "docker"

      config {
        image   = "daprio/daprd:edge"
        ports   = ["http", "grpc", "rpc"]
        command = "./daprd"
        args = [
          "-app-id", "counter-api-dapr-http",
          "-app-port", "${NOMAD_PORT_app}",
          "-dapr-http-port", "${NOMAD_PORT_http}",
          "-config", "local/build/dapr/components/daprConfig.yaml",
          "-resources-path", "local/build/dapr/components",
        ]
      }

      template {
        data        = <<EOF
    {{ key "dapr/daprConfig.yaml" }}
    EOF
        destination = "local/build/dapr/components/daprConfig.yaml"
      }

      template {
        data        = <<EOF
    {{ key "dapr/components/consul.yaml" }}
    EOF
        destination = "local/build/dapr/components/consul.yaml"
      }

      template {
        data        = <<EOF
    {{ key "dapr/components/orderup_pubsub.yaml" }}
    EOF
        destination = "local/build/dapr/components/orderup_pubsub.yaml"
      }

      template {
        data        = <<EOF
    {{ key "dapr/components/barista_pubsub.yaml" }}
    EOF
        destination = "local/build/dapr/components/barista_pubsub.yaml"
      }

      template {
        data        = <<EOF
    {{ key "dapr/components/kitchen_pubsub.yaml" }}
    EOF
        destination = "local/build/dapr/components/kitchen_pubsub.yaml"
      }

      resources {
        memory = 100
      }
    }
  }
}