job "redis" {
  datacenters = ["dc1"]

  group "redis" {
    network {
      mode = "host"

      port "tcp" {
        to     = 6379
        static = 6379
      }
    }

    service {
      name = "redis"
      port = "${NOMAD_PORT_tcp}"

      tags = [
        "addr_ip=${attr.unique.network.ip-address}",
      ]
    }

    task "redis" {
      driver = "docker"

      config {
        image = "redis:alpine"
        ports = ["tcp"]
      }

      env {}
    }
  }
}