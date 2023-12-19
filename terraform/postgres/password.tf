locals {
  db_master_password = var.db_master_password == "" ? random_password.db_master_password[0].result : var.db_master_password
}

resource "random_password" "db_master_password" {
  count   = var.db_master_password == "" ? 1 : 0
  length  = 16
  special = false
}

resource "aws_kms_key" "db_master_password" {
  description         = "KMS key for the ${module.this.id} RDS cluster master password"
  enable_key_rotation = true

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "Enable IAM User Permissions"
        Effect = "Allow"
        Principal = {
          AWS = data.aws_caller_identity.this.account_id
        }
        Action   = "kms:*"
        Resource = "*"
      },
    ]
  })
}

resource "aws_kms_alias" "db_master_password" {
  name          = "alias/${module.this.id}-master-password"
  target_key_id = aws_kms_key.db_master_password.id
}

resource "aws_secretsmanager_secret" "db_master_password" {
  name       = "${module.this.id}-master-password"
  kms_key_id = aws_kms_key.db_master_password.arn
}

resource "aws_secretsmanager_secret_version" "db_master_password" {
  secret_id     = aws_secretsmanager_secret.db_master_password.id
  secret_string = local.db_master_password
}
