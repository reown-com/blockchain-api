module "redis" {
  source  = "./redis"
  context = module.this

  vpc_id      = module.vpc.vpc_id
  subnets_ids = module.vpc.intra_subnets
}
