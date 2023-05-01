job "product-api-dapr" {
  datacenters = ["dc1"]

  group "product-api-dapr" {
    count = 1

    network {
      mode = "bridge"
      port "app" { to = 5001 }
      port "http" { to = 3500 }
      port "grpc" { to = 50001 }
      port "rpc" { to = 40001 }
    }

    service {
      name         = "product-api-dapr-http"
      port         = "${NOMAD_PORT_app}"
      address_mode = "host"

      connect {
        sidecar_service {}
      }

      tags = [
        "dapr",
        // TODO: cannot apply both routers on product-catalog and counter, it might need the reverse proxy on the top of them
        // "traefik.enable=true",
        // "traefik.consulcatalog.connect=true",
        // "traefik.http.routers.api.rule=PathPrefix(`/product-api`)",
        // "traefik.http.routers.api.middlewares=product_api_stripprefix",
        // "traefik.http.middlewares.product_api_stripprefix.stripprefix.prefixes=/product-api",
      ]

      meta {
        DAPR_PORT = "${NOMAD_HOST_PORT_rpc}"
        APP_PORT  = "${NOMAD_PORT_app}"
        ADDR_IP   = "${attr.unique.network.ip-address}"
      }
    }

    task "product-api-dapr" {
      driver = "docker"

      config {
        image = "ghcr.io/thangchung/try-nomad-dapr/productapi:0.1.0"
        ports = ["${NOMAD_PORT_app}"]
      }

      env {
        ASPNETCORE_ENVIRONMENT = "Development"
      }

      resources {
        memory = 128
      }
    }

    task "daprd" {
      driver = "docker"

      config {
        image   = "daprio/daprd:edge"
        ports   = ["http", "grpc", "rpc"]
        command = "./daprd"
        args = [
          "-app-id", "product-api-dapr-http",
          "-app-port", "${NOMAD_PORT_app}",
          "-dapr-http-port", "3500",
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