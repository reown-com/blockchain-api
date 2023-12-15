module "postgres" {
  source = "./postgres"

  app_name            = local.app_name
  vpc_id              = module.vpc.vpc_id
  ingress_cidr_blocks = [module.vpc.vpc_cidr_block]
}
