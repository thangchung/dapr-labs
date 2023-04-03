job "product-api-dapr" {
  datacenters = ["dc1"]

  group "product-api-dapr" {
    network {
      mode = "bridge"
      port "app" { to = 5001 }
      port "http" { to = 3500 }
      port "grpc" { to = 50001 }
      port "rpc" { to = 40001 }
    }

    service {
      name         = "product-api-dapr-http"
      port         = "http"
      address_mode = "host"
      tags = [
        "dapr",
      ]
      meta {
        DAPR_PORT = "${NOMAD_HOST_PORT_rpc}"
      }
    }

    task "product-api-dapr" {
      driver = "docker"

      config {
        image = "ghcr.io/thangchung/try-nomad-dapr/productapi:0.1.0"
        ports = ["app"]
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
    {{ key "build/dapr/components/daprConfig.yaml" }}
    EOF
        destination = "local/build/dapr/components/daprConfig.yaml"
      }

      template {
        data        = <<EOF
    {{ key "build/dapr/components/pubsub.yaml" }}
    EOF
        destination = "local/build/dapr/components/pubsub.yaml"
      }

      template {
        data        = <<EOF
    {{ key "build/dapr/components/consul.yaml" }}
    EOF
        destination = "local/build/dapr/components/consul.yaml"
      }

      resources {
        memory = 128
      }
    }
  }
}