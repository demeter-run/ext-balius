resource "kubernetes_config_map" "cfg" {
  metadata {
    namespace = var.namespace
    name      = "baliusd-${var.network}-config"
  }

  data = {
    "baliusd.toml" = "${templatefile(
      "${path.module}/baliusd.toml.tftpl",
      {
        utxorpc_url    = var.utxorpc_url,
        container_port = local.container_port
      }
    )}"
  }
}
