locals {
  app_name         = "rpc-proxy"
  hosted_zone_name = "rpc.walletconnect.com"
  fqdn             = terraform.workspace == "prod" ? local.hosted_zone_name : "${terraform.workspace}.${local.hosted_zone_name}"
}

# tflint-ignore: terraform_unused_declarations
data "assert_test" "workspace" {
  test  = terraform.workspace != "default"
  throw = "default workspace is not valid in this project"
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

data "aws_ecr_repository" "repository" {
  name = local.app_name
}

# ECS Cluster, Task, Service, and Load Balancer for our app
module "ecs" {
  source = "./ecs"

  ecr_repository_url  = data.aws_ecr_repository.repository.repository_url
  app_name            = "${terraform.workspace}_${local.app_name}"
  region              = var.region
  vpc_name            = "ops-${terraform.workspace}-vpc"
  port                = 3000
  acm_certificate_arn = module.dns.certificate_arn
  fqdn                = local.fqdn
  route53_zone_id     = module.dns.zone_id
  infura_project_id   = var.infura_project_id
  pokt_project_id     = var.pokt_project_id
  prometheus_endpoint = aws_prometheus_workspace.prometheus.prometheus_endpoint

  registry_api_endpoint             = var.registry_api_endpoint
  registry_api_auth_token           = var.registry_api_auth_token
  project_data_cache_ttl            = var.project_data_cache_ttl
  project_data_redis_endpoint_read  = module.redis.endpoint
  project_data_redis_endpoint_write = module.redis.endpoint
}

module "private_hosted_zone" {
  count  = terraform.workspace == "prod" ? 1 : 0
  source = "./private_zone"

  name     = "rpc.repl.internal"
  vpc_name = "ops-${terraform.workspace}-vpc"
}

locals {
  zone_id = terraform.workspace == "prod" ? module.private_hosted_zone.zone_id[0] : null
}

module "redis" {
  source = "./redis"

  redis_name = "rpc-${terraform.workspace}"
  app_name   = local.app_name
  node_type  = "cache.m6g.large"
  vpc_name   = "ops-${terraform.workspace}-vpc"
  zone_id    = local.zone_id
}

module "o11y" {
  source = "./monitoring"

  prometheus_workspace_id = aws_prometheus_workspace.prometheus.id
  environment             = terraform.workspace
}

resource "aws_prometheus_workspace" "prometheus" {
  alias = "prometheus-${terraform.workspace}-${local.app_name}"
}
