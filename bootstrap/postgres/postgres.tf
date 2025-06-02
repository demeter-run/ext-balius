resource "kubernetes_manifest" "postgres" {
  field_manager {
    force_conflicts = true
  }
  manifest = {
    "apiVersion" = "acid.zalan.do/v1"
    "kind"       = "postgresql"
    "metadata" = {
      "name"      = var.name
      "namespace" = var.namespace
    }
    "spec" = {
      "env" : [
        {
          "name" : "ALLOW_NOSSL"
          "value" : "true"
        }
      ]
      "numberOfInstances"         = var.replicas
      "enableMasterLoadBalancer"  = false
      "enableReplicaLoadBalancer" = false
      "allowedSourceRanges" = [
        "0.0.0.0/0"
      ]
      "dockerImage" : "ghcr.io/zalando/spilo-15:3.2-p1"
      "teamId" = "dmtr"
      "tolerations" = [
        {
          "key"      = "demeter.run/workload"
          "operator" = "Equal"
          "value"    = "mem-intensive"
          "effect"   = "NoSchedule"
        },
        {
          "effect"   = "NoSchedule"
          "key"      = "demeter.run/compute-profile"
          "operator" = "Exists"
        },
        {
          "effect"   = "NoSchedule"
          "key"      = "demeter.run/compute-arch"
          "operator" = "Equal"
          "value"    = "arm64"
        },
        {
          "effect"   = "NoSchedule"
          "key"      = "demeter.run/availability-sla"
          "operator" = "Equal"
          "value"    = "consistent"
        }
      ]
      "serviceAnnotations" : {
        "service.beta.kubernetes.io/aws-load-balancer-nlb-target-type" = "instance"
        "service.beta.kubernetes.io/aws-load-balancer-scheme"          = "internet-facing"
        "service.beta.kubernetes.io/aws-load-balancer-type"            = "external"
      }
      "databases" = {
        "balius-cardano-mainnet" = "balius"
        "balius-cardano-preprod" = "balius"
        "balius-cardano-preview" = "balius"
      }
      "postgresql" = {
        "version"    = "14"
        "parameters" = var.params
      }
      "users" = {
        "balius" = [
          "superuser",
          "createdb",
          "login"
        ],
        "dmtrro" = [
          "login"
        ]
      }
      "resources" = {
        "limits"   = var.resources.limits
        "requests" = var.resources.requests
      }
      "volume" = {
        "storageClass" = var.volume.storage_class
        "size"         = var.volume.size
      }
      sidecars = [
        {
          name : "exporter"
          image : "quay.io/prometheuscommunity/postgres-exporter:v0.12.0"
          env : [
            {
              name : "DATA_SOURCE_URI"
              value : "localhost:5432/balius-cardano-mainnet?sslmode=disable"
            },
            {
              name : "DATA_SOURCE_USER"
              value : "$(POSTGRES_USER)"
            },
            {
              name : "DATA_SOURCE_PASS"
              value : "$(POSTGRES_PASSWORD)"
            },
            {
              name : "PG_EXPORTER_CONSTANT_LABELS"
              value : "service=${var.name}"
            }
          ]
          ports : [
            {
              name : "metrics"
              containerPort : 9187
            }
          ]
        }
      ]
    }
  }
}
