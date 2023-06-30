variable "ecr_repository_url" {
  type = string
}

variable "ecr_app_version" {
  description = "The tag of the app image to deploy."
  type        = string
}

variable "app_name" {
  type = string
}

variable "environment" {
  type = string
}

variable "region" {
  type = string
}

variable "port" {
  type = number
}

variable "private_port" {
  type = number
}

variable "acm_certificate_arn" {
  type = string
}

variable "fqdn" {
  type = string
}

variable "route53_zone_id" {
  type = string
}

variable "backup_acm_certificate_arn" {
  type = string
}

variable "backup_fqdn" {
  type = string
}

variable "backup_route53_zone_id" {
  type = string
}

variable "infura_project_id" {
  type = string
}

variable "pokt_project_id" {
  type = string
}

variable "prometheus_endpoint" {
  type = string
}

variable "prometheus_workspace_id" {
  type = string
}

variable "registry_api_endpoint" {
  type = string
}

variable "registry_api_auth_token" {
  type      = string
  sensitive = true
}

variable "project_data_cache_ttl" {
  type = number
}

variable "project_data_redis_endpoint_read" {
  type = string
}

variable "project_data_redis_endpoint_write" {
  type = string
}

variable "identity_cache_redis_endpoint_read" {
  type = string
}

variable "identity_cache_redis_endpoint_write" {
  type = string
}

variable "analytics-data-lake_bucket_name" {
  type = string
}

variable "analytics_geoip_db_key" {
  type = string
}

variable "analytics_geoip_db_bucket_name" {
  type = string
}

variable "analytics_key_arn" {
  type = string
}

variable "autoscaling_max_capacity" {
  type = number
}

variable "autoscaling_min_capacity" {
  type = number
}

variable "public_subnets" {
  type = set(string)
}

variable "private_subnets" {
  type = set(string)
}

variable "vpc_id" {
  type = string
}

variable "vpc_cidr" {
  type = string
}
