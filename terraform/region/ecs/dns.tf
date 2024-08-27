# DNS Records
resource "aws_route53_record" "dns_load_balancer_region" {
  depends_on = [aws_acm_certificate_validation.certificate_validation]
  for_each   = var.route53_zones

  zone_id = each.key
  name    = "${module.this.region}.${each.value}"
  type    = "A"

  alias {
    name                   = aws_lb.load_balancer.dns_name
    zone_id                = aws_lb.load_balancer.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "dns_load_balancer" {
  for_each = var.route53_zones

  zone_id = each.key
  name    = each.value
  type    = "A"

  alias {
    name                   = aws_lb.load_balancer.dns_name
    zone_id                = aws_lb.load_balancer.zone_id
    evaluate_target_health = true
  }

  latency_routing_policy {
    region = var.region
  }
  set_identifier = var.region
}

# resource "aws_route53_health_check" "health_check" {
#   fqdn              = local.fqdn
#   port              = 443
#   type              = "HTTPS"
#   resource_path     = "/health"
#   failure_threshold = "5"
#   request_interval  = "30"

#   tags = {
#     Name = "${var.environment}.${var.region}.${var.app_name}-health-check"
#   }
# }
