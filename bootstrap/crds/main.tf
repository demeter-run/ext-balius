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
          "demeter-port",
        ]
        "kind"   = "BaliusWorker"
        "plural" = "baliusworkers"
        "shortNames" = [
          "sapts",
        ]
        "singular" = "baliusworker"
      }
      "scope" = "Namespaced"
      "versions" = [
        {
          "additionalPrinterColumns" = [
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
                    "authToken" = {
                      "type" = "string"
                    }
                    "config" = {
                      "additionalProperties" = true
                      "type"                 = "object"
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
