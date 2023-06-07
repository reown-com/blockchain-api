variable "environment" {
  description = "The environment name (dev/staging/prod)."
  type        = string
}

variable "prometheus_workspace_id" {
  description = "The workspace ID of the Prometheus workspace."
  type        = string
}

variable "redis_cluster_id" {
  description = "The ID of the Redis cluster."
  type        = string
}

variable "target_group_arn" {
  description = "The ARN of the target group."
  type        = string
}

variable "load_balancer_arn" {
  description = "The ARN of the load balancer."
  type        = string
}
