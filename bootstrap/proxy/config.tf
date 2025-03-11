// numbers here should consider number of proxy replicas
locals {
  tiers = [
    {
      "name" = "0",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(1 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(80000 / var.replicas)
        }
      ]
    },
    {
      "name" = "1",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(5 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(400000 / var.replicas)
        }
      ]
    },
    {
      "name" = "2",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(40 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(2000000 / var.replicas)
        }
      ]
    },
    {
      "name" = "3",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(80 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(4000000 / var.replicas)
        }
      ]
    }
  ]
}

resource "kubernetes_config_map" "proxy" {
  metadata {
    namespace = var.namespace
    name      = "proxy-config"
  }

  data = {
    "tiers.toml" = "${templatefile("${path.module}/proxy-config.toml.tftpl", { tiers = local.tiers })}"
  }
}
