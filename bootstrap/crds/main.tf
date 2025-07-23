resource "kubernetes_manifest" "customresourcedefinition_baliusworkers_demeter_run" {
  manifest = {
    "apiVersion" = "apiextensions.k8s.io/v1"
    "kind"       = "CustomResourceDefinition"
    "metadata" = {
      "name" = "baliusworkers.demeter.run"
    }
    "spec" = {
      "group" = "demeter.run"
      "names" = {
        "categories" = [
          "demeter-worker",
        ]
        "kind"   = "BaliusWorker"
        "plural" = "baliusworkers"
        "shortNames" = [
          "bwapts",
        ]
        "singular" = "baliusworker"
      }
      "scope" = "Namespaced"
      "versions" = [
        {
          "additionalPrinterColumns" = [
            {
              "jsonPath" = ".spec.active"
              "name"     = "Active"
              "type"     = "boolean"
            },
            {
              "jsonPath" = ".spec.displayName"
              "name"     = "Display Name"
              "type"     = "string"
            },
            {
              "jsonPath" = ".spec.network"
              "name"     = "Network"
              "type"     = "string"
            },
            {
              "jsonPath" = ".spec.throughputTier"
              "name"     = "Throughput Tier"
              "type"     = "string"
            },
            {
              "jsonPath" = ".status.endpointUrl"
              "name"     = "Endpoint URL"
              "type"     = "string"
            },
            {
              "jsonPath" = ".status.authenticatedEndpointUrl"
              "name"     = "Authenticated Endpoint URL"
              "type"     = "string"
            },
            {
              "jsonPath" = ".status.authToken"
              "name"     = "Auth Token"
              "type"     = "string"
            },
          ]
          "name" = "v1alpha1"
          "schema" = {
            "openAPIV3Schema" = {
              "description" = "Auto-generated derived type for BaliusWorkerSpec via `CustomResource`"
              "properties" = {
                "spec" = {
                  "properties" = {
                    "active" = {
                      "nullable" = true
                      "type"     = "boolean"
                    }
                    "authToken" = {
                      "type" = "string"
                    }
                    "config" = {
                      "additionalProperties" = true
                      "type"                 = "object"
                    }
                    "displayName" = {
                      "type" = "string"
                    }
                    "network" = {
                      "type" = "string"
                    }
                    "throughputTier" = {
                      "type" = "string"
                    }
                    "url" = {
                      "type" = "string"
                    }
                    "version" = {
                      "type" = "string"
                    }
                  }
                  "required" = [
                    "authToken",
                    "config",
                    "displayName",
                    "network",
                    "throughputTier",
                    "url",
                    "version",
                  ]
                  "type" = "object"
                }
                "status" = {
                  "nullable" = true
                  "properties" = {
                    "authToken" = {
                      "type" = "string"
                    }
                    "authenticatedEndpointUrl" = {
                      "nullable" = true
                      "type"     = "string"
                    }
                    "endpointUrl" = {
                      "type" = "string"
                    }
                    "error" = {
                      "nullable" = true
                      "type"     = "string"
                    }
                  }
                  "required" = [
                    "authToken",
                    "endpointUrl",
                  ]
                  "type" = "object"
                }
              }
              "required" = [
                "spec",
              ]
              "title" = "BaliusWorker"
              "type"  = "object"
            }
          }
          "served"  = true
          "storage" = true
          "subresources" = {
            "status" = {}
          }
        },
      ]
    }
  }
}
