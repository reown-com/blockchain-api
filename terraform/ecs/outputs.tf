output "ecs_cluster_name" {
  description = "The name of the ECS cluster"
  value       = aws_ecs_cluster.app_cluster.name
}

output "ecs_service_name" {
  description = "The name of the ECS service"
  value       = aws_ecs_service.app_service.name
}

output "ecs_task_family" {
  description = "The family of the task definition"
  value       = aws_ecs_task_definition.app_task.family
}

output "service_security_group_id" {
  description = "The ID of the security group for the service"
  value       = aws_security_group.app_ingress.id
}

output "target_group_arn" {
  description = "The ARN of the target group"
  value       = aws_lb_target_group.target_group.arn
}

output "load_balancer_arn" {
  description = "The ARN of the load balancer"
  value       = aws_lb.load_balancer.arn
}

output "load_balancer_arn_suffix" {
  description = "The ARN suffix of the load balancer"
  value       = aws_lb.load_balancer.arn_suffix
}

output "log_group_app_name" {
  description = "The name of the log group for the app"
  value       = aws_cloudwatch_log_group.cluster.name
}

output "log_group_app_arn" {
  description = "The ARN of the log group for the app"
  value       = aws_cloudwatch_log_group.cluster.arn
}
