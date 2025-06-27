locals {
  vault_kms_key_alias   = "vault-autounseal"
  vault_kms_policy_name = "demeter-vault-kms-policy"
  vault_kms_user_name   = "demeter-vault-kms-user"
}

resource "helm_release" "vault" {
  name       = "vault"
  namespace  = var.namespace
  repository = var.vault_chart_repository
  chart      = var.vault_chart

  values = [
    templatefile("${path.module}/values.yml.tftpl", {
      aws_region                    = var.aws_region,
      vault_credentials_secret_name = var.vault_credentials_secret_name,
      aws_kms_key_id                = aws_kms_key.vault_autounseal.arn
      vault_cert_secret_name        = var.vault_cert_secret_name
    })
  ]
}

resource "aws_kms_key" "vault_autounseal" {
  description             = "KMS key for Vault auto-unseal"
  deletion_window_in_days = var.vault_kms_key_deletion_window_days
}

resource "aws_kms_alias" "vault_autounseal_alias" {
  name          = "alias/${local.vault_kms_key_alias}"
  target_key_id = aws_kms_key.vault_autounseal.key_id
}

resource "aws_iam_policy" "vault_kms_policy" {
  name        = local.vault_kms_policy_name
  description = "Policy to allow Vault auto-unseal via AWS KMS"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowVaultAutoUnseal"
        Effect = "Allow"
        Action = [
          "kms:Decrypt",
          "kms:Encrypt",
          "kms:GenerateDataKey*",
          "kms:DescribeKey",
        ]
        Resource = aws_kms_key.vault_autounseal.arn
      }
    ]
  })
}

resource "aws_iam_user" "vault_kms_user" {
  name = local.vault_kms_user_name
}

resource "aws_iam_user_policy_attachment" "vault_kms_user_policy" {
  user       = aws_iam_user.vault_kms_user.name
  policy_arn = aws_iam_policy.vault_kms_policy.arn
}

resource "aws_iam_access_key" "vault_kms_user_keys" {
  user = aws_iam_user.vault_kms_user.name
}

resource "kubernetes_secret" "vault_kms_credentials" {
  metadata {
    name      = var.vault_credentials_secret_name
    namespace = var.namespace
  }

  type = "Opaque"

  data = {
    aws_region            = var.aws_region
    aws_access_key_id     = aws_iam_access_key.vault_kms_user_keys.id
    aws_secret_access_key = aws_iam_access_key.vault_kms_user_keys.secret
  }
}
