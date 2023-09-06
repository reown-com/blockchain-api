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

variable "zerion_api_key" {
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

variable "autoscaling_max_capacity" {
  type = number
}

variable "autoscaling_min_capacity" {
  type = number
}

variable "private_route_table_ids" {
  type = set(string)
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

variable "analytics_data_lake_bucket_name" {
  description = "The name of the data-lake bucket."
  type        = string
}

variable "analytics_data_lake_kms_key_arn" {
  description = "The ARN of the KMS encryption key for data-lake bucket."
  type        = string
}

variable "analytics_geoip_db_key" {
  description = "The key to the GeoIP database"
  type        = string
  default     = "GeoLite2-City.mmdb"
}

variable "analytics_geoip_db_bucket_name" {
  description = "The name of the bucket containing the GeoIP database"
  type        = string
}
