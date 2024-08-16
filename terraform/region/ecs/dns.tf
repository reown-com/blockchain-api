# DNS Records
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
}
