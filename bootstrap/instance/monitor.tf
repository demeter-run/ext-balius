resource "kubernetes_manifest" "podmonitor" {
  manifest = {
    "apiVersion" = "monitoring.coreos.com/v1"
    "kind"       = "PodMonitor"
    "metadata" = {
      "labels" = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      "name"      = local.name
      "namespace" = var.namespace
    }
    "spec" = {
      podMetricsEndpoints = [
        {
          port = "metrics",
          path = "/metrics"
        }
      ]
      "selector" = {
        "matchLabels" = {
          "demeter.run/instance"        = local.name
          "cardano.demeter.run/network" = var.network
        }
      }
    }
  }
}
