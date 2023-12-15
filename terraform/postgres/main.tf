module "db_cluster" {
  source  = "terraform-aws-modules/rds-aurora/aws"
  version = "8.5.0"

  name           = "${var.app_name}-postgres-cluster"
  engine         = "aurora-postgresql"
  engine_version = "16.1"
  engine_mode    = "provisioned"
  instance_class = "db.serverless"
  instances      = { for i in range(1, var.instances + 1) : i => {} }

  database_name               = var.db_name
  manage_master_user_password = false
  master_username             = var.db_master_username
  master_password             = local.db_master_password

  vpc_id = var.vpc_id
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

  monitoring_interval = 30

  serverlessv2_scaling_configuration = {
    min_capacity = var.min_capacity
    max_capacity = var.max_capacity
  }
}
