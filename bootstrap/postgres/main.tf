variable "namespace" {
  type = string
}

variable "name" {
  type = string
}

variable "resources" {
  type = object({
    requests = map(string)
    limits   = map(string)
  })

  default = {
    "limits" = {
      memory = "4Gi"
      cpu    = "4000m"
    }
    "requests" = {
      memory = "4Gi"
      cpu    = "100m"
    }
  }
}

variable "params" {
  default = {
    "max_standby_archive_delay"   = "900s"
    "max_standby_streaming_delay" = "900s"
  }
}

variable "volume" {
  type = object({
    storage_class = string
    size          = string
  })
  default = {
    storage_class = "fast"
    size          = "30Gi"
  }
}

variable "replicas" {
  type    = number
  default = 2
}
