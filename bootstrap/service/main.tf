locals {
  service_name = "balius-${var.network}"
  port         = 3000
}

variable "namespace" {
  description = "The namespace where the resources will be created"
}

variable "network" {
  description = "Cardano node network"
}

resource "kubernetes_service_v1" "well_known_service" {
  metadata {
    name      = local.service_name
    namespace = var.namespace
  }

  spec {
    port {
      name     = "api"
      protocol = "TCP"
      port     = local.port
    }

    selector = {
      "cardano.demeter.run/network" = var.network
    }

    type = "ClusterIP"
  }
}
