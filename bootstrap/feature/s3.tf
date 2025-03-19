locals {
  workers_bucket_name       = "demeter-workers"
  workers_bucket_iam_policy = "demeter-workers-policy"
  workers_bucket_iam_user   = "demeter-workers-user"
  workers_secret_name       = "demeter-workers-credentials"
}

resource "aws_s3_bucket" "workers_storage" {
  bucket = local.workers_bucket_name

  force_destroy = true
}

resource "aws_s3_bucket_ownership_controls" "workers_storage_ownership_controls" {
  bucket = aws_s3_bucket.workers_storage.id
  rule {
    object_ownership = "ObjectWriter"
  }
}

resource "aws_s3_bucket_public_access_block" "workers_storage_public_block" {
  bucket = aws_s3_bucket.workers_storage.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

## IAM Policy to upload on Workers Storage Bucket
resource "aws_iam_policy" "api_upload_workers_storage_policy" {
  name        = local.workers_bucket_iam_policy
  description = "Policy to upload, read and delete files from the demeter workers bucket"

  policy = jsonencode({
    "Version" : "2012-10-17",
    "Statement" : [
      {
        "Sid" : "VisualEditor0",
        "Effect" : "Allow",
        "Action" : [
          "s3:PutObject",
          "s3:GetObjectAcl",
          "s3:GetObject",
          "s3:AbortMultipartUpload",
          "s3:DeleteObjectVersion",
          "s3:RestoreObject",
          "s3:GetObjectVersionAcl",
          "s3:DeleteObject",
          "s3:PutObjectAcl",
          "s3:GetObjectVersion"
        ],
        "Resource" : "${aws_s3_bucket.workers_storage.arn}/*",
      }
    ]
  })
}

## IAM User to upload on Demeter Workers bucket
resource "aws_iam_user" "api_upload_workers_storage_user" {
  name = local.workers_bucket_iam_user
}

resource "aws_iam_user_policy_attachment" "attach_iam_user_workers_storage_policy" {
  user       = aws_iam_user.api_upload_workers_storage_user.name
  policy_arn = aws_iam_policy.api_upload_workers_storage_policy.arn
}

resource "aws_iam_access_key" "iam_user_workers_storage_keys" {
  user = aws_iam_user.api_upload_workers_storage_user.name
}


resource "kubernetes_secret" "workers_s3_credentials" {
  metadata {
    name      = local.workers_secret_name
    namespace = var.namespace
  }

  type = "Opaque"

  data = {
    aws_access_key_id     = aws_iam_access_key.iam_user_workers_storage_keys.id
    aws_secret_access_key = aws_iam_access_key.iam_user_workers_storage_keys.secret
  }
}
