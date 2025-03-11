locals {
  role = "operator"
  port = 9946
}

resource "kubernetes_deployment_v1" "operator" {
  wait_for_rollout = false

  metadata {
    namespace = var.namespace
    name      = local.role
    labels = {
      role = local.role
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        role = local.role
      }
    }

    template {
      metadata {
        labels = {
          role = local.role
        }
      }

      spec {
        container {
          image = "ghcr.io/demeter-run/ext-balius-operator:${var.operator_image_tag}"
          name  = "main"

          env {
            name  = "ADDR"
            value = "0.0.0.0:${local.port}"
          }

          env {
            name  = "K8S_IN_CLUSTER"
            value = "true"
          }

          env {
            name  = "PROMETHEUS_URL"
            value = "http://prometheus-operated.demeter-system.svc.cluster.local:9090/api/v1"
          }

          env {
            name  = "METRICS_DELAY"
            value = var.metrics_delay
          }

          env {
            name  = "EXTENSION_DOMAIN"
            value = var.extension_domain
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
            name           = "metrics"
            container_port = local.port
            protocol       = "TCP"
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
