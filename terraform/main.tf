locals {
  app_name                = "rpc-proxy"
  hosted_zone_name        = "rpc.walletconnect.com"
  backup_hosted_zone_name = "rpc.walletconnect.org"
  fqdn                    = terraform.workspace == "prod" ? local.hosted_zone_name : "${terraform.workspace}.${local.hosted_zone_name}"
  backup_fqdn             = terraform.workspace == "prod" ? local.backup_hosted_zone_name : "${terraform.workspace}.${local.backup_hosted_zone_name}"

  analytics_geoip_db_bucket_name  = "${terraform.workspace}.relay.geo.ip.database.private.${terraform.workspace}.walletconnect"
  analytics_data_lake_bucket_name = "walletconnect.data-lake.${terraform.workspace}"
}

# tflint-ignore: terraform_unused_declarations
data "assert_test" "workspace" {
  test  = terraform.workspace != "default"
  throw = "default workspace is not valid in this project"
}

#tfsec:ignore:aws-ec2-require-vpc-flow-logs-for-all-vpcs
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.0.0"
  name    = "${terraform.workspace}-${local.app_name}"

  cidr = "10.0.0.0/16"

  azs             = var.azs
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.4.0/24", "10.0.5.0/24", "10.0.6.0/24"]

  private_subnet_tags = {
    Visibility = "private"
  }
  public_subnet_tags = {
    Visibility = "public"
  }

  enable_dns_support     = true
  enable_dns_hostnames   = true
  enable_nat_gateway     = true
  single_nat_gateway     = true
  one_nat_gateway_per_az = false
}

module "tags" {
  # tflint-ignore: terraform_module_pinned_source
  source = "github.com/WalletConnect/terraform-modules/modules/tags"

  application = local.app_name
  env         = terraform.workspace
}

module "dns" {
  # tflint-ignore: terraform_module_pinned_source
  source = "github.com/WalletConnect/terraform-modules/modules/dns"

  hosted_zone_name = local.hosted_zone_name
  fqdn             = local.fqdn
}

module "backup_dns" {
  # tflint-ignore: terraform_module_pinned_source
  source = "github.com/WalletConnect/terraform-modules/modules/dns"

  hosted_zone_name = local.backup_hosted_zone_name
  fqdn             = local.backup_fqdn
}

data "aws_ecr_repository" "repository" {
  name = local.app_name
}

module "logging" {
  source   = "./logging"
  app_name = local.app_name
}

# ECS Cluster, Task, Service, and Load Balancer for our app
module "ecs" {
  source = "./ecs"

  environment = terraform.workspace

  ecr_repository_url         = data.aws_ecr_repository.repository.repository_url
  ecr_app_version            = var.ecr_app_version
  app_name                   = "${terraform.workspace}_${local.app_name}"
  region                     = var.region
  private_subnets            = module.vpc.private_subnets
  public_subnets             = module.vpc.public_subnets
  vpc_cidr                   = module.vpc.vpc_cidr_block
  vpc_id                     = module.vpc.vpc_id
  private_route_table_ids    = module.vpc.private_route_table_ids
  port                       = 3000
  private_port               = 4000
  acm_certificate_arn        = module.dns.certificate_arn
  fqdn                       = local.fqdn
  route53_zone_id            = module.dns.zone_id
  backup_acm_certificate_arn = module.backup_dns.certificate_arn
  backup_fqdn                = local.backup_fqdn
  backup_route53_zone_id     = module.backup_dns.zone_id
  infura_project_id          = var.infura_project_id
  pokt_project_id            = var.pokt_project_id
  zerion_api_key             = var.zerion_api_key
  prometheus_endpoint        = aws_prometheus_workspace.prometheus.prometheus_endpoint
  prometheus_workspace_id    = aws_prometheus_workspace.prometheus.id

  autoscaling_min_capacity = var.autoscaling_min_instances
  autoscaling_max_capacity = var.autoscaling_max_instances

  registry_api_endpoint               = var.registry_api_endpoint
  registry_api_auth_token             = var.registry_api_auth_token
  project_data_cache_ttl              = var.project_data_cache_ttl
  project_data_redis_endpoint_read    = module.redis.endpoint
  project_data_redis_endpoint_write   = module.redis.endpoint
  identity_cache_redis_endpoint_read  = module.redis.endpoint
  identity_cache_redis_endpoint_write = module.redis.endpoint

  analytics_data_lake_bucket_name = local.analytics_data_lake_bucket_name
  analytics_data_lake_kms_key_arn = var.analytics_data_lake_kms_key_arn
  analytics_geoip_db_bucket_name  = local.analytics_geoip_db_bucket_name
}

module "redis" {
  source = "./redis"

  redis_name      = "rpc-${terraform.workspace}"
  app_name        = local.app_name
  node_type       = "cache.t4g.micro" # https://aws.amazon.com/elasticache/pricing/?nc=sn&loc=5#On-demand_nodes
  private_subnets = module.vpc.private_subnets
  vpc_cidr        = module.vpc.vpc_cidr_block
  vpc_id          = module.vpc.vpc_id
}

module "monitoring" {
  source = "./monitoring"

  prometheus_workspace_id = aws_prometheus_workspace.prometheus.id
  environment             = terraform.workspace
  redis_cluster_id        = module.redis.cluster_id
  target_group_arn        = module.ecs.target_group_arn
  load_balancer_arn       = module.ecs.load_balancer_arn
}

resource "aws_prometheus_workspace" "prometheus" {
  alias = "prometheus-${terraform.workspace}-${local.app_name}"
}
