terraform {
  required_version = "~> 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0.0"
    }
  }
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
  subnet_ids = var.private_subnets
}

# Allow only the app to access Redis
resource "aws_security_group" "service_security_group" {
  name        = "${var.app_name}-${var.redis_name}-redis-service-ingress"
  description = "Allow ingress from the application"
  vpc_id      = var.vpc_id
  ingress {
    description = "${var.app_name}-${var.redis_name} - ingress from application"
    from_port   = 6379
    to_port     = 6379
    protocol    = "TCP"
    cidr_blocks = var.allowed_ingress_cidr_blocks == null ? [var.vpc_cidr] : var.allowed_ingress_cidr_blocks
  }

  egress {
    description = "${var.app_name}-${var.redis_name} - egress to application"
    from_port   = 0    # Allowing any incoming port
    to_port     = 0    # Allowing any outgoing port
    protocol    = "-1" # Allowing any outgoing protocol
    cidr_blocks = var.allowed_egress_cidr_blocks == null ? [var.vpc_cidr] : var.allowed_egress_cidr_blocks
  }
}

locals {
  cache_endpoint = "${aws_elasticache_cluster.cache.cache_nodes[0].address}:${aws_elasticache_cluster.cache.cache_nodes[0].port}"
}
