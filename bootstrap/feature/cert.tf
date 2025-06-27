resource "kubernetes_manifest" "vault_cert" {
  manifest = {
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata" = {
      "name"      = var.vault_cert_secret_name
      "namespace" = var.namespace
    }
    "spec" = {
      // DNS names are not verified by clients. Used only for secure communication intra cluster.
      "dnsNames" = [
        "vault.demeter.run"
      ]

      "issuerRef" = {
        "kind" = "ClusterIssuer"
        "name" = "letsencrypt-dns01"
      }
      "secretName" = var.vault_cert_secret_name
    }
  }
}

