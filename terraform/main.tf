data "aws_caller_identity" "this" {}

resource "random_pet" "this" {
  length = 2
}

locals {
  ecr_repository_url = local.is_dev ? data.terraform_remote_state.org.outputs.accounts.sdlc.dev.ecr-urls.blockchain : data.terraform_remote_state.org.outputs.accounts.wl.blockchain[local.stage].ecr-url

  stage = lookup({
    "blockchain-wl-staging" = "staging",
    "blockchain-wl-prod"    = "prod",
    "blockchain-wl-dev"     = "dev",
    "blockchain-staging"    = "staging",
    "blockchain-prod"       = "prod",
    "wl-staging"            = "staging",
    "wl-prod"               = "prod",
    "wl-dev"                = "dev",
    "staging"               = "staging",
    "prod"                  = "prod",
  }, terraform.workspace, terraform.workspace)

  is_dev     = local.stage == "dev"     #tflint-ignore: terraform_unused_declarations
  is_staging = local.stage == "staging" #tflint-ignore: terraform_unused_declarations
  is_prod    = local.stage == "prod"    #tflint-ignore: terraform_unused_declarations
}

resource "aws_kms_key" "cloudwatch_logs" {
  description         = "KMS key for encrypting CloudWatch Logs"
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
      {
        Sid    = "AllowCloudWatchLogs"
        Effect = "Allow"
        Principal = {
          Service = "logs.${module.this.region}.amazonaws.com"
        }
        Action = [
          "kms:Encrypt*",
          "kms:Decrypt*",
          "kms:ReEncrypt*",
          "kms:GenerateDataKey*",
          "kms:Describe*"
        ]
        Resource = "*"
      },
    ]
  })
}

resource "aws_kms_alias" "cloudwatch_logs" {
  name          = "alias/${module.this.id}-cloudwatch-logs"
  target_key_id = aws_kms_key.cloudwatch_logs.key_id
}
