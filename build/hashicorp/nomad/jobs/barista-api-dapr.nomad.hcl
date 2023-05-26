job "barista-api-dapr" {
  datacenters = ["dc1"]

  group "barista-api-dapr" {
    network {
      mode = "bridge"
      port "app" { to = 5003 }
      port "http" { to = 3500 }
      port "grpc" { to = 50001 }
      port "rpc" { to = 40001 }
    }

    service {
      name         = "barista-api-dapr-http"
      port         = "${NOMAD_PORT_app}"
      address_mode = "host"

      connect {
        sidecar_service {}
      }

      tags = [
        "dapr"
      ]
      meta {
        DAPR_PORT = "${NOMAD_HOST_PORT_rpc}"
        APP_PORT  = "${NOMAD_PORT_app}"
      }
    }

    task "barista-api-dapr" {
      driver = "docker"

      config {
        image = "ghcr.io/thangchung/try-nomad-dapr/baristaapi:0.1.0"
        ports = ["${NOMAD_PORT_app}"]
      }

      env {
        ASPNETCORE_ENVIRONMENT       = "Development"
        ConnectionStrings__baristadb = "Server=${attr.unique.network.ip-address};Port=5432;Database=postgres;User Id=postgres;Password=P@ssw0rd"
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
          "-app-id", "barista-api-dapr-http",
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