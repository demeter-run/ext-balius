locals {
  postgres_name = "balius-postgres"
  postgres_host = "${local.postgres_name}.${var.namespace}.svc.cluster.local"
}

resource "kubernetes_namespace" "namespace" {
  metadata {
    name = var.namespace
  }
}

module "feature" {
  depends_on              = [kubernetes_namespace.namespace]
  source                  = "./feature"
  namespace               = var.namespace
  operator_image_tag      = var.operator_image_tag
  metrics_delay           = var.metrics_delay
  resources               = var.operator_resources
  tolerations             = var.operator_tolerations
  extension_domain        = var.extension_domain
  credentials_secret_name = "demeter-workers-credentials"
}

module "proxy" {
  depends_on      = [kubernetes_namespace.namespace]
  source          = "./proxy"
  proxy_image_tag = var.proxy_image_tag
  namespace       = var.namespace
  replicas        = var.proxy_replicas
  resources       = var.proxy_resources
  dns_names       = var.dns_names
  tolerations     = var.proxy_tolerations
}

module "postgres" {
  depends_on = [kubernetes_namespace.namespace]
  source     = "./postgres"

  name      = local.postgres_name
  namespace = var.namespace
  resources = var.postgres_resources
  params    = var.postgres_params
  volume    = var.postgres_volume
  replicas  = var.postgres_replicas
  networks  = var.networks
}

module "instances" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = var.instances
  source     = "./instance"

  namespace               = var.namespace
  image                   = each.value.image
  salt                    = each.value.salt
  network                 = each.value.network
  utxorpc_url             = each.value.utxorpc_url
  vault_token             = each.value.vault_token
  vault_address           = each.value.vault_address
  replicas                = coalesce(each.value.replicas, 1)
  credentials_secret_name = "demeter-workers-credentials"
  postgres_name           = local.postgres_name
  postgres_host           = local.postgres_host
  resources = coalesce(each.value.resources, {
    limits : {
      cpu : "200m",
      memory : "1Gi"
    }
    requests : {
      cpu : "200m",
      memory : "500Mi"
    }
  })
  tolerations = coalesce(each.value.tolerations, [
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-profile"
      operator = "Exists"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-arch"
      operator = "Exists"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/availability-sla"
      operator = "Exists"
    }
  ])
}

module "services" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = { for network in var.networks : "${network}" => network }
  source     = "./service"

  namespace = var.namespace
  network   = each.value
}

