job "product-api-dapr" {
  datacenters = ["dc1"]

  constraint {
    attribute = "${attr.kernel.name}"
    value     = "linux"
  }

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
      driver = "raw_exec"

      artifact {
        source      = "git::https://github.com/vietnam-devs/go-coffeeshop"
        destination = "local/repo"
      }

      config {
        command = "bash"
        args = [
          "-c",
          "git checkout feature/dapr && cd local/repo/cmd/product && go mod tidy && go mod download && CGO_ENABLED=0 go run github.com/thangchung/go-coffeeshop/cmd/product"
        ]
      }

      env {
        APP_NAME = "product-service in docker"
      }

    }

    task "daprd" {
      driver = "docker"

      config {
        image   = "daprio/daprd:latest"
        ports   = ["http", "grpc", "rpc"]
        command = "daprd"
        args = [
          "-app-id", "product-api-dapr-http",
          "-app-port", "${NOMAD_PORT_app}",
          "-dapr-http-port", "3500",
          "-config", "local/build/dapr/components/daprConfig.yaml",
          "-resources-path", "local/build/dapr/components",
        ]
      }

      template {
        data        = <<EOH
{{ key "dapr/config.yaml" }}
EOH
        destination = "local/build/dapr/components/daprConfig.yaml"
      }

      template {
        data        = <<EOH
{{ key "dapr/components/pubsub.yaml" }}
EOH
        destination = "local/build/dapr/components/pubsub.yaml"
      }

      template {
        data        = <<EOH
{{ key "dapr/components/consul.yaml" }}
EOH
        destination = "local/build/dapr/components/consul.yaml"
      }

      resources {
        memory = 128
      }
    }
  }
}