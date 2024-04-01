#-------------------------------------------------------------------------------
# Configuration

variable "grafana_auth" {
  description = "The API Token for the Grafana instance"
  type        = string
  default     = ""
}

#-------------------------------------------------------------------------------
# Application

variable "name" {
  description = "The name of the application"
  type        = string
  default     = "blockchain-api"
}

variable "region" {
  description = "AWS region to deploy to"
  type        = string
}

variable "image_version" {
  description = "The ECS tag of the image to deploy"
  type        = string
}

variable "log_level" {
  description = "Defines logging level for the application"
  type        = string
}

variable "app_autoscaling_desired_count" {
  description = "The desired number of tasks to run"
  type        = number
  default     = 2
}

variable "app_autoscaling_min_capacity" {
  description = "The minimum number of tasks to run when autoscaling"
  type        = number
  default     = 2
}

variable "app_autoscaling_max_capacity" {
  description = "The maximum number of tasks to run when autoscaling"
  type        = number
  default     = 8
}

variable "ofac_blocked_countries" {
  description = "The list of countries to block"
  type        = string
  default     = ""
}

#-------------------------------------------------------------------------------
# Project Registry

variable "registry_api_endpoint" {
  description = "The endpoint of the registry API"
  type        = string
}

variable "registry_api_auth_token" {
  description = "The auth token for the registry API"
  type        = string
  sensitive   = true
}

variable "project_cache_ttl" {
  description = "The TTL for project data cache"
  type        = number
  default     = 300
}


#-------------------------------------------------------------------------------
# Providers

variable "infura_project_id" {
  description = "The project ID for Infura"
  type        = string
  sensitive   = true
}

variable "pokt_project_id" {
  description = "The project ID for POKT"
  type        = string
  sensitive   = true
}

variable "zerion_api_key" {
  description = "The API key for Zerion"
  type        = string
  sensitive   = true
}

variable "quicknode_api_token" {
  description = "The API key for Quicknode"
  type        = string
  sensitive   = true
}

variable "coinbase_api_key" {
  description = "The API key for Coinbase Pay SDK"
  type        = string
  sensitive   = true
}

variable "coinbase_app_id" {
  description = "The APP-ID for Coinbase Pay SDK"
  type        = string
  sensitive   = true
}

variable "one_inch_api_key" {
  description = "The API key for 1inch"
  type        = string
  sensitive   = true
}

variable "getblock_access_tokens" {
  description = "Mapping of API access tokens for GetBlock in JSON format"
  type        = string
  sensitive   = true
}

variable "testing_project_id" {
  description = "Project ID used in a testing suite"
  type        = string
  sensitive   = true
}

#-------------------------------------------------------------------------------
# Analytics

variable "geoip_db_key" {
  description = "The name to the GeoIP database"
  type        = string
}

#-------------------------------------------------------------------------------
# Alerting / Monitoring

variable "notification_channels" {
  description = "The notification channels to send alerts to"
  type        = list(any)
  default     = []
}

variable "webhook_cloudwatch_p2" {
  description = "The webhook to send CloudWatch P2 alerts to"
  type        = string
  default     = ""
}

variable "webhook_prometheus_p2" {
  description = "The webhook to send Prometheus P2 alerts to"
  type        = string
  default     = ""
}

#-------------------------------------------------------------------------------
# Rate-limiting (Token bucket) configuration

variable "rate_limiting_max_tokens" {
  description = "The maximum number of tokens in the bucket"
  type        = number
  default     = 100
}

variable "rate_limiting_refill_interval" {
  description = "The interval in seconds to refill the bucket"
  type        = number
  default     = 1
}

variable "rate_limiting_refill_rate" {
  description = "The number of tokens to refill the bucket with"
  type        = number
  default     = 2
}
