data "aws_caller_identity" "this" {}

resource "aws_db_subnet_group" "db_subnets" {
  name        = module.this.id
  description = "Subnet group for the ${module.this.id} RDS cluster"
  subnet_ids  = var.subnet_ids
}

module "db_cluster" {
  source  = "terraform-aws-modules/rds-aurora/aws"
  version = "8.5.0"

  name               = module.this.id
  database_name      = var.db_name
  engine             = "aurora-postgresql"
  engine_version     = "15.5"
  engine_mode        = "provisioned"
  ca_cert_identifier = "rds-ca-ecc384-g1"
  instance_class     = "db.serverless"
  instances          = { for i in range(1, var.instances + 1) : i => {} }

  master_username             = var.db_master_username
  manage_master_user_password = false
  master_password             = local.db_master_password

  vpc_id               = var.vpc_id
  db_subnet_group_name = aws_db_subnet_group.db_subnets.name
  security_group_rules = {
    vpc_ingress = {
      cidr_blocks = var.ingress_cidr_blocks
    }
  }

  performance_insights_enabled = true
  storage_encrypted            = true
  allow_major_version_upgrade  = true
  apply_immediately            = true
  skip_final_snapshot          = true
  deletion_protection          = true

  monitoring_interval                    = 30
  enabled_cloudwatch_logs_exports        = ["postgresql"]
  cloudwatch_log_group_kms_key_id        = var.cloudwatch_logs_key_arn
  cloudwatch_log_group_retention_in_days = var.cloudwatch_retention_in_days

  serverlessv2_scaling_configuration = {
    min_capacity = module.this.stage == "prod" ? var.min_capacity : 0.5
    max_capacity = var.max_capacity
  }
}
