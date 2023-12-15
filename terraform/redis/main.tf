data "aws_vpc" "vpc" {
  id = var.vpc_id
}

resource "aws_elasticache_cluster" "cache" {
  cluster_id           = module.this.id
  engine               = "redis"
  node_type            = var.node_type
  num_cache_nodes      = var.num_cache_nodes
  parameter_group_name = "default.redis6.x"
  engine_version       = var.node_engine_version
  port                 = 6379
  subnet_group_name    = aws_elasticache_subnet_group.private_subnets.name
  security_group_ids = [
    aws_security_group.service_security_group.id
  ]
  snapshot_retention_limit = 2
}

resource "aws_elasticache_subnet_group" "private_subnets" {
  name       = "${module.this.id}-private-subnet-group"
  subnet_ids = var.subnets_ids
}

# Allow only the app to access Redis
resource "aws_security_group" "service_security_group" {
  name        = "${module.this.id}-redis-service-ingress"
  description = "Allow ingress from the application"
  vpc_id      = var.vpc_id
  ingress {
    description = "${module.this.id} - ingress from application"
    from_port   = 6379
    to_port     = 6379
    protocol    = "TCP"
    cidr_blocks = var.ingress_cidr_blocks == null ? [data.aws_vpc.vpc.cidr_block] : var.ingress_cidr_blocks
  }

  egress {
    description = "${module.this.id} - egress to application"
    from_port   = 0    # Allowing any incoming port
    to_port     = 0    # Allowing any outgoing port
    protocol    = "-1" # Allowing any outgoing protocol
    cidr_blocks = var.egress_cidr_blocks == null ? [data.aws_vpc.vpc.cidr_block] : var.egress_cidr_blocks
  }
}
