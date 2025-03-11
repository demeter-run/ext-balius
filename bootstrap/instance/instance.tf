locals {
  name           = "balius-${var.network}-${var.salt}"
  container_port = 3000
}

resource "kubernetes_deployment_v1" "balius" {
  wait_for_rollout = false

  metadata {
    name      = local.name
    namespace = var.namespace
    labels = {
      "demeter.run/kind"            = "BaliusInstance"
      "cardano.demeter.run/network" = var.network
    }
  }

  spec {
    replicas = var.replicas

    strategy {
      rolling_update {
        max_surge       = 1
        max_unavailable = 0
      }
    }
    selector {
      match_labels = {
        "demeter.run/instance"        = local.name
        "cardano.demeter.run/network" = var.network
      }
    }

    template {

      metadata {
        name = local.name
        labels = {
          "demeter.run/instance"        = local.name
          "cardano.demeter.run/network" = var.network
        }
      }

      spec {
        restart_policy = "Always"

        security_context {
          fs_group = 1000
        }

        container {
          name              = "main"
          image             = var.image
          image_pull_policy = "IfNotPresent"

          env {
            name  = "BALIUSD_CONFIG"
            value = "/etc/config/baliusd.toml"
          }

          volume_mount {
            name       = "config"
            mount_path = "/etc/config"
          }

          resources {
            limits = {
              cpu    = var.resources.limits.cpu
              memory = var.resources.limits.memory
            }
            requests = {
              cpu    = var.resources.requests.cpu
              memory = var.resources.requests.memory
            }
          }

          port {
            container_port = local.container_port
            name           = "api"
          }
        }

        volume {
          name = "config"
          config_map {
            name = "baliusd-${var.network}-config"
          }
        }

        dynamic "toleration" {
          for_each = var.tolerations
          content {
            effect   = toleration.value.effect
            key      = toleration.value.key
            operator = toleration.value.operator
            value    = toleration.value.value
          }
        }
      }
    }
  }
}

