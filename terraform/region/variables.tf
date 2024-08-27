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

variable "quicknode_api_tokens" {
  description = "API keys for Quicknode in JSON format"
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

variable "one_inch_referrer" {
  description = "The referrer address for 1inch"
  type        = string
  sensitive   = true
}

variable "getblock_access_tokens" {
  description = "Mapping of API access tokens for GetBlock in JSON format"
  type        = string
  sensitive   = true
}

variable "pimlico_api_key" {
  description = "Pimlico bundler API token key"
  type        = string
  sensitive   = true
}

variable "solscan_api_v1_token" {
  description = "Solscan API v1 token"
  type        = string
  sensitive   = true
}

variable "solscan_api_v2_token" {
  description = "Solscan API v2 token"
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
}

variable "webhook_cloudwatch_p2" {
  description = "The webhook to send CloudWatch P2 alerts to"
  type        = string
}

variable "webhook_prometheus_p2" {
  description = "The webhook to send Prometheus P2 alerts to"
  type        = string
}

#-------------------------------------------------------------------------------
# Rate-limiting (Token bucket) configuration

variable "rate_limiting_max_tokens" {
  description = "The maximum number of tokens in the bucket"
  type        = number
  default     = 30
}

variable "rate_limiting_refill_interval" {
  description = "The interval in seconds to refill the bucket"
  type        = number
  default     = 1
}

variable "rate_limiting_refill_rate" {
  description = "The number of tokens to refill the bucket with"
  type        = number
  default     = 3
}

#-------------------------------------------------------------------------------
# IRN VPC peering

variable "irn_vpc_id" {
  description = "ID of the IRN VPC"
  type        = string
}

variable "irn_vpc_cidr" {
  description = "CIDR block of the IRN VPC"
  type        = string
}

variable "irn_aws_account_id" {
  description = "ID of the AWS account in IRN is being deployed"
  type        = string
}

#-------------------------------------------------------------------------------
# IRN client configuration

variable "irn_node" {
  description = "IRN node address in Address:Socket format"
  type        = string
}

variable "irn_key" {
  description = "IRN client key in base64 format"
  type        = string
}

variable "irn_namespace" {
  description = "IRN storage namespace"
  type        = string
}

variable "irn_namespace_secret" {
  description = "IRN storage namespace secret key"
  type        = string
}

# Postgres VPC peering

variable "database_url" {
  description = "The URL used to connect to the cluster"
  type        = string
  default     = null
}

variable "database_vpc_id" {
  description = "ID of the database VPC"
  type        = string
  default     = null
}

variable "database_vpc_cidr" {
  description = "CIDR block of the database VPC"
  type        = string
  default     = null
}

variable "database_vpc_region" {
  description = "Region of the database VPC"
  type        = string
  default     = null
}

variable "database_client_vpc_peering_connections" {
  description = "Peering connections of database client VPCs"
  type        = map(string)
  default     = {}
}
