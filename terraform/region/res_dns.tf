locals {
  zones = { for k, v in tomap(data.terraform_remote_state.infra_aws.outputs.zones.blockchain[local.stage]) : v.id => v.name }
  zones_certificates = concat(
    [for k, v in module.dns_certificate : v.certificate_arn],
    [for k, v in module.dns_certificate_region : v.certificate_arn]
  )
}

module "dns_certificate_region" {
  for_each         = local.zones
  source           = "app.terraform.io/wallet-connect/dns/aws"
  version          = "0.1.3"
  context          = module.this
  hosted_zone_name = each.value
  fqdn             = "${module.this.region}.${each.value}"
}

module "dns_certificate" {
  for_each         = local.zones
  source           = "app.terraform.io/wallet-connect/dns/aws"
  version          = "0.1.3"
  context          = module.this
  hosted_zone_name = each.value
  fqdn             = each.value
}
