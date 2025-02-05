data "aws_s3_bucket" "geoip" {
  bucket = data.terraform_remote_state.infra_aws.outputs.geoip_bucked_id
}

resource "aws_prometheus_workspace" "prometheus" {
  alias = "prometheus-${module.this.id}"
}

resource "aws_iam_role" "application_role" {
  name = "${module.this.id}-ecs-task-execution"
  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })
}

# ECS Cluster, Task, Service, and Load Balancer for our app
module "ecs" {
  source  = "./ecs"
  context = module.this

  # Cluster
  ecr_repository_url        = local.ecr_repository_url
  image_version             = var.image_version
  task_execution_role_name  = aws_iam_role.application_role.name
  task_cpu                  = 2048
  task_memory               = 8192
  autoscaling_desired_count = var.app_autoscaling_desired_count
  autoscaling_min_capacity  = var.app_autoscaling_min_capacity
  autoscaling_max_capacity  = var.app_autoscaling_max_capacity
  cloudwatch_logs_key_arn   = aws_kms_key.cloudwatch_logs.arn

  # DNS
  route53_zones              = local.zones
  route53_zones_certificates = local.zones_certificates

  # Network
  vpc_id                          = module.vpc.vpc_id
  public_subnets                  = module.vpc.public_subnets
  private_subnets                 = module.vpc.private_subnets
  allowed_app_ingress_cidr_blocks = module.vpc.vpc_cidr_block
  allowed_lb_ingress_cidr_blocks  = module.vpc.vpc_cidr_block

  # Application
  port                               = 8080
  log_level                          = var.log_level
  project_cache_endpoint_read        = module.redis.endpoint
  project_cache_endpoint_write       = module.redis.endpoint
  identity_cache_endpoint_read       = module.redis.endpoint
  identity_cache_endpoint_write      = module.redis.endpoint
  rate_limiting_cache_endpoint_read  = module.redis.endpoint
  rate_limiting_cache_endpoint_write = module.redis.endpoint
  provider_cache_endpoint            = module.redis.endpoint
  ofac_blocked_countries             = var.ofac_blocked_countries
  postgres_url                       = module.postgres.database_url

  # Providers
  infura_project_id      = var.infura_project_id
  pokt_project_id        = var.pokt_project_id
  quicknode_api_tokens   = var.quicknode_api_tokens
  zerion_api_key         = var.zerion_api_key
  coinbase_api_key       = var.coinbase_api_key
  coinbase_app_id        = var.coinbase_app_id
  one_inch_api_key       = var.one_inch_api_key
  one_inch_referrer      = var.one_inch_referrer
  getblock_access_tokens = var.getblock_access_tokens
  pimlico_api_key        = var.pimlico_api_key
  solscan_api_v2_token   = var.solscan_api_v2_token
  bungee_api_key         = var.bungee_api_key
  lava_api_key           = var.lava_api_key
  tenderly_api_key       = var.tenderly_api_key
  tenderly_account_id    = var.tenderly_account_id
  tenderly_project_id    = var.tenderly_project_id
  dune_api_key           = var.dune_api_key
  syndica_api_key        = var.syndica_api_key
  allnodes_api_key       = var.allnodes_api_key

  # Project Registry
  registry_api_endpoint   = var.registry_api_endpoint
  registry_api_auth_token = var.registry_api_auth_token
  project_cache_ttl       = var.project_cache_ttl

  # Rate Limiting
  rate_limiting_max_tokens      = var.rate_limiting_max_tokens
  rate_limiting_refill_interval = var.rate_limiting_refill_interval
  rate_limiting_refill_rate     = var.rate_limiting_refill_rate
  rate_limiting_ip_whitelist    = var.rate_limiting_ip_whitelist

  # IRN Client
  irn_nodes            = var.irn_nodes
  irn_key              = var.irn_key
  irn_namespace        = var.irn_namespace
  irn_namespace_secret = var.irn_namespace_secret

  # ENS Names
  names_allowed_zones = var.names_allowed_zones

  # Address balances related configuration
  balances_denylist_project_ids = var.balances_denylist_project_ids

  # Analytics
  analytics_datalake_bucket_name = data.terraform_remote_state.datalake.outputs.datalake_bucket_id
  analytics_datalake_kms_key_arn = data.terraform_remote_state.datalake.outputs.datalake_kms_key_arn

  # Monitoring
  prometheus_workspace_id = aws_prometheus_workspace.prometheus.id
  prometheus_endpoint     = aws_prometheus_workspace.prometheus.prometheus_endpoint

  # GeoIP
  geoip_db_bucket_name = data.aws_s3_bucket.geoip.id
  geoip_db_key         = var.geoip_db_key

  # Project ID used in a testing suite
  testing_project_id = var.testing_project_id

  depends_on = [aws_iam_role.application_role]
}
