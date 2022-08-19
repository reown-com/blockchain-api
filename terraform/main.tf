locals {
  app_name = "rpc-proxy"
}

data "assert_test" "workspace" {
  test  = terraform.workspace != "default"
  throw = "default workspace is not valid in this project"
}

module "tags" {
  source = "github.com/WalletConnect/terraform-modules/modules/tags"

  application = local.app_name
  env         = terraform.workspace
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
}
