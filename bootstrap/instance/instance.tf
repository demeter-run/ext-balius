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

          env {
            name = "POSTGRES_USER"
            value_from {
              secret_key_ref {
                key  = "username"
                name = "balius.${var.postgres_name}.credentials.postgresql.acid.zalan.do"
              }
            }
          }

          env {
            name = "POSTGRES_PASSWORD"
            value_from {
              secret_key_ref {
                key  = "password"
                name = "balius.${var.postgres_name}.credentials.postgresql.acid.zalan.do"
              }
            }
          }

          env {
            name  = "POSTGRES_HOST"
            value = var.postgres_host
          }

          env {
            name  = "BALIUSD_CONNECTION"
            value = "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):5432/${replace(var.network, "-", "")}"
          }

          env {
            name = "BALIUSD_POD"
            value_from {
              field_ref {
                field_path = "metadata.name"
              }
            }
          }

          env {
            name = "BALIUSD_NAMESPACE"
            value_from {
              field_ref {
                field_path = "metadata.namespace"
              }
            }
          }

          env {
            name = "AWS_REGION"
            value_from {
              secret_key_ref {
                name = var.credentials_secret_name
                key  = "aws_region"
              }
            }
          }

          env {
            name = "AWS_ACCESS_KEY_ID"
            value_from {
              secret_key_ref {
                name = var.credentials_secret_name
                key  = "aws_access_key_id"
              }
            }
          }

          env {
            name = "AWS_SECRET_ACCESS_KEY"
            value_from {
              secret_key_ref {
                name = var.credentials_secret_name
                key  = "aws_secret_access_key"
              }
            }
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

