#-------------------------------------------------------------------------------
# Cluster

variable "ecr_repository_url" {
  description = "The URL of the ECR repository where the app image is stored"
  type        = string
}

variable "image_version" {
  description = "The version of the app image to deploy"
  type        = string
}

variable "task_execution_role_name" {
  description = "The name of the task execution role"
  type        = string
}

variable "task_cpu" {
  description = "The number of CPU units to reserve for the container."
  type        = number
}

variable "task_memory" {
  description = "The amount of memory (in MiB) to reserve for the container."
  type        = number
}

variable "autoscaling_desired_count" {
  description = "Minimum number of instances in the autoscaling group"
  type        = number
  default     = 2
}

variable "autoscaling_min_capacity" {
  description = "Minimum number of instances in the autoscaling group"
  type        = number
  default     = 2
}

variable "autoscaling_max_capacity" {
  description = "Maximum number of instances in the autoscaling group"
  type        = number
  default     = 8
}

variable "cloudwatch_logs_key_arn" {
  description = "The ARN of the KMS key to use for encrypting CloudWatch logs"
  type        = string
}

variable "cloudwatch_retention_in_days" {
  description = "The number of days to retain CloudWatch logs for the DB instance"
  type        = number
  default     = 14
}

#-------------------------------------------------------------------------------
# DNS

variable "route53_zones" {
  description = "The FQDNs to use for the app"
  type        = map(string)
}

variable "route53_zones_certificates" {
  description = "The ARNs of the ACM certificates to use for HTTPS"
  type        = map(string)
}

#-------------------------------------------------------------------------------
# Network

variable "vpc_id" {
  description = "The ID of the VPC to deploy to"
  type        = string
}

variable "public_subnets" {
  description = "The IDs of the public subnets"
  type        = list(string)
}

variable "private_subnets" {
  description = "The IDs of the private subnets"
  type        = list(string)
}

variable "allowed_app_ingress_cidr_blocks" {
  description = "A list of CIDR blocks to allow ingress access to the application."
  type        = string
}

variable "allowed_lb_ingress_cidr_blocks" {
  description = "A list of CIDR blocks to allow ingress access to the load-balancer."
  type        = string
}

#-------------------------------------------------------------------------------
# Application

variable "port" {
  description = "The port the app listens on"
  type        = number
}

#tflint-ignore: terraform_unused_declarations
variable "log_level" {
  description = "The log level for the app"
  type        = string
}

variable "redis_max_connections" {
  description = "The maximum number of connections to the Redis server"
  type        = number
  default     = 512
}

variable "project_cache_endpoint_read" {
  description = "The endpoint of the project cache (read)"
  type        = string
}

variable "project_cache_endpoint_write" {
  description = "The endpoint of the project cache (write)"
  type        = string
}

variable "identity_cache_endpoint_read" {
  description = "The endpoint of the identity cache (read)"
  type        = string
}

variable "identity_cache_endpoint_write" {
  description = "The endpoint of the identity cache (write)"
  type        = string
}

variable "rate_limiting_cache_endpoint_read" {
  description = "The endpoint of the rate limiting cache (read)"
  type        = string
}

variable "rate_limiting_cache_endpoint_write" {
  description = "The endpoint of the rate limiting cache (write)"
  type        = string
}

variable "provider_cache_endpoint" {
  description = "Non-RPC providers responses caching endpoint"
  type        = string
}

variable "ofac_countries" {
  description = "The list of countries under OFAC sanctions"
  type        = string
}

variable "postgres_url" {
  description = "The connection URL for the PostgreSQL instance"
  type        = string
}

#-------------------------------------------------------------------------------
# Providers

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

variable "coinbase_api_key_id" {
  description = "The API key ID for Coinbase API"
  type        = string
  sensitive   = true
}

variable "coinbase_api_key_secret" {
  description = "The API key secret for Coinbase API"
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

variable "pimlico_api_key" {
  description = "Pimlico bundler API token key"
  type        = string
  sensitive   = true
}

variable "solscan_api_v2_token" {
  description = "Solscan API v2 token"
  type        = string
  sensitive   = true
}

variable "bungee_api_key" {
  description = "Bungee API key"
  type        = string
  sensitive   = true
}

variable "tenderly_api_key" {
  description = "Tenderly API key"
  type        = string
  sensitive   = true
}

variable "tenderly_account_id" {
  description = "Tenderly Account ID"
  type        = string
  sensitive   = true
}

variable "tenderly_project_id" {
  description = "Tenderly Project ID"
  type        = string
  sensitive   = true
}

variable "dune_api_key" {
  description = "Dune API key"
  type        = string
  sensitive   = true
}

variable "syndica_api_key" {
  description = "Syndica API key"
  type        = string
  sensitive   = true
}

variable "allnodes_api_key" {
  description = "Allnodes API key"
  type        = string
  sensitive   = true
}

variable "meld_api_key" {
  description = "Meld API key"
  type        = string
  sensitive   = true
}

variable "meld_api_url" {
  description = "Meld API base URL. e.g. https://api.meld.io"
  type        = string
  sensitive   = true
}

variable "callstatic_api_key" {
  description = "Callstatic API key"
  type        = string
  sensitive   = true
}

variable "blast_api_key" {
  description = "Blast API key"
  type        = string
  sensitive   = true
}

variable "testing_project_id" {
  description = "Project ID used in a testing suite"
  type        = string
  sensitive   = true
}

#-------------------------------------------------------------------------------
# RPC Proxy configuration
variable "proxy_skip_quota_chains" {
  description = "Comma separated list of CAIP-2 chains to skip quota check"
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
}

#-------------------------------------------------------------------------------
# Analytics

variable "analytics_datalake_bucket_name" {
  description = "The name of the S3 bucket to use for the analytics datalake"
  type        = string
}

variable "analytics_datalake_kms_key_arn" {
  description = "The ARN of the KMS key to use with the datalake bucket"
  type        = string
}

#-------------------------------------------------------------------------------
# Autoscaling

variable "autoscaling_cpu_target" {
  description = "The target CPU utilization for the autoscaling group"
  type        = number
  default     = 50
}

variable "autoscaling_cpu_scale_in_cooldown" {
  description = "The cooldown period (in seconds) before a scale in is possible"
  type        = number
  default     = 180
}

variable "autoscaling_cpu_scale_out_cooldown" {
  description = "The cooldown period (in seconds) before a scale out is possible"
  type        = number
  default     = 180
}

variable "autoscaling_memory_target" {
  description = "The target memory utilization for the autoscaling group"
  type        = number
  default     = 50
}

variable "autoscaling_memory_scale_in_cooldown" {
  description = "The cooldown period (in seconds) before a scale in is possible"
  type        = number
  default     = 180
}

variable "autoscaling_memory_scale_out_cooldown" {
  description = "The cooldown period (in seconds) before a scale out is possible"
  type        = number
  default     = 180
}

#-------------------------------------------------------------------------------
# Monitoring

variable "prometheus_endpoint" {
  description = "The endpoint of the Prometheus server to use for monitoring"
  type        = string
}

variable "prometheus_workspace_id" {
  description = "The workspace ID of the Prometheus server used for monitoring"
  type        = string
}

#---------------------------------------
# GeoIP

variable "geoip_db_bucket_name" {
  description = "The name of the S3 bucket where the GeoIP database is stored"
  type        = string
}

variable "geoip_db_key" {
  description = "The key of the GeoIP database in the S3 bucket"
  type        = string
}

#-------------------------------------------------------------------------------
# Rate-limiting (Token bucket) configuration

variable "rate_limiting_max_tokens" {
  description = "The maximum number of tokens in the bucket"
  type        = number
}

variable "rate_limiting_refill_interval" {
  description = "The interval in seconds to refill the bucket"
  type        = number
}

variable "rate_limiting_refill_rate" {
  description = "The number of tokens to refill the bucket with"
  type        = number
}

variable "rate_limiting_ip_whitelist" {
  description = "Comma separated list of whitelisted IPs"
  type        = string
}

#-------------------------------------------------------------------------------
# IRN client configuration

variable "irn_nodes" {
  description = "Comma-separated IRN nodes address in MultiAddr format"
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

#-------------------------------------------------------------------------------
# Names configuration

variable "names_allowed_zones" {
  description = "Comma separated list of allowed zones for names"
  type        = string
}

#-------------------------------------------------------------------------------
# Address balances projects denylist
variable "balances_denylist_project_ids" {
  description = "Comma separated list of project IDs to denylist"
  type        = string
}

#-------------------------------------------------------------------------------
# Exchanges

variable "coinbase_project_id" {
  description = "Coinbase project id"
  type        = string
  default     = ""
}

variable "coinbase_key_name" {
  description = "Coinbase key name"
  type        = string
  default     = ""
}

variable "coinbase_key_secret" {
  description = "Coinbase key secret"
  type        = string
  sensitive   = true
  default     = ""
}

variable "binance_client_id" {
  description = "Binance client id"
  type        = string
  sensitive   = true
  default     = ""
}

variable "binance_token" {
  description = "Binance token"
  type        = string
  sensitive   = true
  default     = ""
}

variable "binance_key" {
  description = "Binance key"
  type        = string
  sensitive   = true
  default     = ""
}

variable "binance_host" {
  description = "Binance host"
  type        = string
  sensitive   = true
  default     = ""
}

variable "pay_allowed_project_ids" {
  description = "Allowed project ids for pay with exchange"
  type        = string
  sensitive   = false
  default     = ""
}