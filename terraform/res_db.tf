module "db_context" {
  source  = "app.terraform.io/wallet-connect/label/null"
  version = "0.3.2"
  context = module.this

  attributes = [
    "db"
  ]
}

module "postgres" {
  source     = "./postgres"
  context    = module.db_context
  attributes = ["postgres"]

  vpc_id              = module.vpc.vpc_id
  subnet_ids          = module.vpc.intra_subnets
  ingress_cidr_blocks = module.vpc.private_subnets_cidr_blocks

  cloudwatch_logs_key_arn = aws_kms_key.cloudwatch_logs.arn

  depends_on = [aws_iam_role.application_role]
}
