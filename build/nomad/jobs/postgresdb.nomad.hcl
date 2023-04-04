job "postgres-db" {
  datacenters = ["dc1"]

  group "postgres-db" {
    network {
      mode = "bridge"

      port "postgres_db" {
        static = 5432
      }
    }

    service {
      name = "postgres-db"
      port = "5432"

      tags = [
        "addr_ip=${attr.unique.network.ip-address}",
      ]
    }

    task "postgres-db" {
      driver = "docker"

      config {
        image = "postgres:14-alpine"
        ports = ["postgres_db"]
      }

      env {
        POSTGRES_USER     = "postgres"
        POSTGRES_PASSWORD = "P@ssw0rd"
        POSTGRES_DB       = "postgres"
      }
    }
  }
}