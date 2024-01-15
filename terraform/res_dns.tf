locals {
  zones              = { for k, v in tomap(data.terraform_remote_state.infra_aws.outputs.zones.blockchain[local.stage]) : v.id => v.name }
  zones_certificates = { for k, v in module.dns_certificate : v.zone_id => v.certificate_arn }
}

module "dns_certificate" {
  for_each         = local.zones
  source           = "app.terraform.io/wallet-connect/dns/aws"
  version          = "0.1.3"
  context          = module.this
  hosted_zone_name = each.value
  fqdn             = each.value
}
