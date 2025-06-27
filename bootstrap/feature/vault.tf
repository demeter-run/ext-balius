resource "helm_release" "vault" {
  name       = "vault"
  namespace  = var.namespace
  repository = var.vault_chart_repository
  chart      = var.vault_chart

  values = [
    yamlencode({
      server = {
        tolerations = var.vault_tolerations
      }
    })
  ]
}
