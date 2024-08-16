data "aws_caller_identity" "this" {}

resource "random_pet" "this" {
  length = 2
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

module "eu-central-1" {
  source                        = "./region"
  region                        = "eu-central-1"
  image_version                 = var.image_version
  log_level                     = var.log_level
  app_autoscaling_desired_count = var.app_autoscaling_desired_count
  app_autoscaling_min_capacity  = var.app_autoscaling_min_capacity
  app_autoscaling_max_capacity  = var.app_autoscaling_max_capacity
  ofac_blocked_countries        = var.ofac_blocked_countries
  registry_api_endpoint         = var.registry_api_endpoint
  registry_api_auth_token       = var.registry_api_auth_token
  project_cache_ttl             = var.project_cache_ttl
  infura_project_id             = var.infura_project_id
  pokt_project_id               = var.pokt_project_id
  zerion_api_key                = var.zerion_api_key
  quicknode_api_tokens          = var.quicknode_api_tokens
  coinbase_api_key              = var.coinbase_api_key
  coinbase_app_id               = var.coinbase_app_id
  one_inch_api_key              = var.one_inch_api_key
  one_inch_referrer             = var.one_inch_referrer
  getblock_access_tokens        = var.getblock_access_tokens
  pimlico_api_key               = var.pimlico_api_key
  testing_project_id            = var.testing_project_id
  geoip_db_key                  = var.geoip_db_key
  rate_limiting_max_tokens      = var.rate_limiting_max_tokens
  rate_limiting_refill_interval = var.rate_limiting_refill_interval
  rate_limiting_refill_rate     = var.rate_limiting_refill_rate
  irn_vpc_id                    = var.irn_vpc_ids["eu-central-1"]
  irn_vpc_cidr                  = var.irn_vpc_cidrs["eu-central-1"]
  irn_aws_account_id            = var.irn_aws_account_id
  irn_node                      = var.irn_nodes["eu-central-1"]
  irn_key                       = var.irn_key
  irn_namespace                 = var.irn_namespace
  irn_namespace_secret          = var.irn_namespace_secret

  providers = {
    aws = aws.eu-central-1
  }
}

module "us-east-1" {
  source                        = "./region"
  region                        = "us-east-1"
  image_version                 = var.image_version
  log_level                     = var.log_level
  app_autoscaling_desired_count = var.app_autoscaling_desired_count
  app_autoscaling_min_capacity  = var.app_autoscaling_min_capacity
  app_autoscaling_max_capacity  = var.app_autoscaling_max_capacity
  ofac_blocked_countries        = var.ofac_blocked_countries
  registry_api_endpoint         = var.registry_api_endpoint
  registry_api_auth_token       = var.registry_api_auth_token
  project_cache_ttl             = var.project_cache_ttl
  infura_project_id             = var.infura_project_id
  pokt_project_id               = var.pokt_project_id
  zerion_api_key                = var.zerion_api_key
  quicknode_api_tokens          = var.quicknode_api_tokens
  coinbase_api_key              = var.coinbase_api_key
  coinbase_app_id               = var.coinbase_app_id
  one_inch_api_key              = var.one_inch_api_key
  one_inch_referrer             = var.one_inch_referrer
  getblock_access_tokens        = var.getblock_access_tokens
  pimlico_api_key               = var.pimlico_api_key
  testing_project_id            = var.testing_project_id
  geoip_db_key                  = var.geoip_db_key
  rate_limiting_max_tokens      = var.rate_limiting_max_tokens
  rate_limiting_refill_interval = var.rate_limiting_refill_interval
  rate_limiting_refill_rate     = var.rate_limiting_refill_rate
  irn_vpc_id                    = var.irn_vpc_ids["us-east-1"]
  irn_vpc_cidr                  = var.irn_vpc_cidrs["us-east-1"]
  irn_aws_account_id            = var.irn_aws_account_id
  irn_node                      = var.irn_nodes["us-east-1"]
  irn_key                       = var.irn_key
  irn_namespace                 = var.irn_namespace
  irn_namespace_secret          = var.irn_namespace_secret

  providers = {
    aws = aws.us-east-1
  }
}

module "ap-southeast-1" {
  source                        = "./region"
  region                        = "ap-southeast-1"
  image_version                 = var.image_version
  log_level                     = var.log_level
  app_autoscaling_desired_count = var.app_autoscaling_desired_count
  app_autoscaling_min_capacity  = var.app_autoscaling_min_capacity
  app_autoscaling_max_capacity  = var.app_autoscaling_max_capacity
  ofac_blocked_countries        = var.ofac_blocked_countries
  registry_api_endpoint         = var.registry_api_endpoint
  registry_api_auth_token       = var.registry_api_auth_token
  project_cache_ttl             = var.project_cache_ttl
  infura_project_id             = var.infura_project_id
  pokt_project_id               = var.pokt_project_id
  zerion_api_key                = var.zerion_api_key
  quicknode_api_tokens          = var.quicknode_api_tokens
  coinbase_api_key              = var.coinbase_api_key
  coinbase_app_id               = var.coinbase_app_id
  one_inch_api_key              = var.one_inch_api_key
  one_inch_referrer             = var.one_inch_referrer
  getblock_access_tokens        = var.getblock_access_tokens
  pimlico_api_key               = var.pimlico_api_key
  testing_project_id            = var.testing_project_id
  geoip_db_key                  = var.geoip_db_key
  rate_limiting_max_tokens      = var.rate_limiting_max_tokens
  rate_limiting_refill_interval = var.rate_limiting_refill_interval
  rate_limiting_refill_rate     = var.rate_limiting_refill_rate
  irn_vpc_id                    = var.irn_vpc_ids["ap-southeast-1"]
  irn_vpc_cidr                  = var.irn_vpc_cidrs["ap-southeast-1"]
  irn_aws_account_id            = var.irn_aws_account_id
  irn_node                      = var.irn_nodes["ap-southeast-1"]
  irn_key                       = var.irn_key
  irn_namespace                 = var.irn_namespace
  irn_namespace_secret          = var.irn_namespace_secret

  providers = {
    aws = aws.ap-southeast-1
  }
}
