variable "webhook_cloudwatch_p2" {
  description = "The URL of the webhook to be called on CloudWatch P2 alarms"
  type        = string
}

variable "webhook_prometheus_p2" {
  description = "The URL of the webhook to be called on Prometheus P2 alarms"
  type        = string
}

#-------------------------------------------------------------------------------
# ECS

variable "ecs_cluster_name" {
  description = "The name of the ECS cluster running the application"
  type        = string
}

variable "ecs_service_name" {
  description = "The name of the ECS service running the application"
  type        = string
}

variable "ecs_cpu_threshold" {
  description = "The ECS CPU utilization alarm threshold in percents"
  type        = number
  default     = 80
}

variable "ecs_memory_threshold" {
  description = "The ECS memory utilization alarm threshold in percents"
  type        = number
  default     = 80
}

#-------------------------------------------------------------------------------
# Redis

variable "redis_cluster_id" {
  description = "The Redis cluster ID"
  type        = string
}

variable "redis_cpu_threshold" {
  description = "The Redis CPU utilization alarm threshold in percents"
  type        = number
  default     = 80
}

variable "redis_memory_threshold" {
  description = "The Redis available memory alarm threshold in GiB"
  type        = number
  default     = 3
}
