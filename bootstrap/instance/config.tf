resource "kubernetes_config_map" "cfg" {
  metadata {
    namespace = var.namespace
    name      = "baliusd-${var.network}-config"
  }

  data = {
    "baliusd.toml" = "${templatefile(
      "${path.module}/baliusd.toml.tftpl",
      {
        utxorpc_url                = var.utxorpc_url,
        container_port             = local.container_port
        prometheus_port            = local.prometheus_port
        network                    = var.network
        vault_address              = var.vault_address
        vault_token                = var.vault_token
        vault_token_renew_seconds  = var.vault_token_renew_seconds
        vault_token_renew_interval = var.vault_token_renew_interval
      }
    )}"
  }
}
