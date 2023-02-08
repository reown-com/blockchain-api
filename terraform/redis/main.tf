terraform {
  required_version = "~> 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.4"
    }
  }
}

data "aws_vpc" "vpc" {
  filter {
    name   = "tag:Name"
    values = [var.vpc_name]
  }
}

# Providing a reference to our default subnets
data "aws_subnets" "private_subnets" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.vpc.id]
  }

  filter {
    name   = "tag:Class"
    values = ["private"]
  }
}

locals {
  zone_name = var.zone_name == null ? "local" : var.zone_name
}

resource "aws_elasticache_cluster" "cache" {
  cluster_id           = replace("${var.app_name}-${var.redis_name}", "_", "-")
  engine               = "redis"
  node_type            = var.node_type
  num_cache_nodes      = 1
  parameter_group_name = "default.redis6.x"
  engine_version       = "6.x"
  port                 = 6379
  subnet_group_name    = aws_elasticache_subnet_group.private_subnets.name
  security_group_ids = [
    aws_security_group.service_security_group.id
  ]
  snapshot_retention_limit = 2
}

resource "aws_elasticache_subnet_group" "private_subnets" {
  name       = replace("${var.app_name}-${var.redis_name}-private-subnet-group", "_", "-")
  subnet_ids = data.aws_subnets.private_subnets.ids
}

# Allow only the app to access Redis
resource "aws_security_group" "service_security_group" {
  name        = "${var.app_name}-${var.redis_name}-redis-service-ingress"
  description = "Allow ingress from the application"
  vpc_id      = data.aws_vpc.vpc.id
  ingress {
    description = "${var.app_name}-${var.redis_name} - ingress from application"
    from_port   = 6379
    to_port     = 6379
    protocol    = "TCP"
    cidr_blocks = var.allowed_ingress_cidr_blocks == null ? [data.aws_vpc.vpc.cidr_block] : var.allowed_ingress_cidr_blocks
  }

  egress {
    description = "${var.app_name}-${var.redis_name} - egress to application"
    from_port   = 0    # Allowing any incoming port
    to_port     = 0    # Allowing any outgoing port
    protocol    = "-1" # Allowing any outgoing protocol
    cidr_blocks = var.allowed_egress_cidr_blocks == null ? [data.aws_vpc.vpc.cidr_block] : var.allowed_egress_cidr_blocks
  }
}

# DNS
resource "aws_route53_record" "dns" {
  count   = terraform.workspace == "prod" ? 1 : 0
  zone_id = var.zone_id
  name    = "${replace("${var.redis_name}-redis", "_", "-")}.${local.zone_name}"
  type    = "CNAME"
  ttl     = "30"
  records = [for cache_node in aws_elasticache_cluster.cache.cache_nodes : cache_node.address]
}

locals {
  cache_endpoint = "${aws_elasticache_cluster.cache.cache_nodes[0].address}:${aws_elasticache_cluster.cache.cache_nodes[0].port}"
}
