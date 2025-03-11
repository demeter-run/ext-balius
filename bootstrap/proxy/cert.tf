resource "kubernetes_manifest" "certificate_cluster_wildcard_tls" {
  manifest = {
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata" = {
      "name"      = var.cert_secret_name
      "namespace" = var.namespace
    }
    "spec" = {
      "dnsNames" = var.dns_names

      "issuerRef" = {
        "kind" = "ClusterIssuer"
        "name" = "letsencrypt-dns01"
      }
      "secretName" = var.cert_secret_name
    }
  }
}

