variable "namespace" {
  type = string
}

variable "extension_domain" {
  type = string
}

variable "operator_image_tag" {
  type = string
}

variable "metrics_delay" {
  description = "The inverval for polling metrics data (in seconds)"
  default     = "30"
}

variable "credentials_secret_name" {
  type = string
}

variable "tolerations" {
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = [
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
      operator = "Equal"
      value    = "consistent"
    }
  ]
}

variable "resources" {
  type = object({
    limits = object({
      cpu    = string
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    limits = {
      cpu    = "1"
      memory = "512Mi"
    }
    requests = {
      cpu    = "50m"
      memory = "512Mi"
    }
  }
}

variable "vault_chart" {
  description = "Name of the Vault Helm chart"
  type        = string
  default     = "vault"
}

variable "vault_chart_repository" {
  description = "Helm repository URL for the Vault chart"
  type        = string
  default     = "https://helm.releases.hashicorp.com"
}

variable "vault_tolerations" {
  description = "Tolerations for the Vault server pods"
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = [
    {
      effect   = "NoSchedule"
      key      = "demeter.run/availability-sla"
      operator = "Equal"
      value    = "consistent"
    }
  ]
}
