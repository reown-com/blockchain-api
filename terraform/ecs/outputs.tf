output "service_security_group_id" {
  value = aws_security_group.vpc_app_ingress.id
}

output "target_group_arn" {
  value = aws_lb_target_group.target_group.arn
}

output "load_balancer_arn" {
  value = aws_alb.network_load_balancer.arn
}
