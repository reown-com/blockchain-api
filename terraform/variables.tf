#-------------------------------------------------------------------------------
# Configuration

variable "grafana_auth" {
  description = "The API Token for the Grafana instance"
  type        = string
  default     = ""
  sensitive   = true
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
  default     = 3
}

variable "app_autoscaling_min_capacity" {
  description = "The minimum number of tasks to run when autoscaling"
  type        = number
  default     = 3
}

variable "app_autoscaling_max_capacity" {
  description = "The maximum number of tasks to run when autoscaling"
  type        = number
  default     = 10
}

variable "ofac_countries" {
  description = "The list of countries to block"
  type        = string
  default     = ""
}

variable "ecr_repository_name" {
  description = "Name of the ECR repository"
  type        = string
  default     = "blockchain"
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

variable "registry_circuit_cooldown_ms" {
  description = "Circuit breaker cooldown in milliseconds for registry failures"
  type        = number
  default     = 1000
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

variable "dune_sim_api_key" {
  description = "Dune Sim API key"
  type        = string
  sensitive   = true
}

variable "syndica_api_key" {
  description = "Syndica API key"
  type        = string
  sensitive   = true
}

variable "testing_project_id" {
  description = "Project ID used in a testing suite"
  type        = string
  sensitive   = true
}

variable "validate_project_id" {
  description = "Project ID and quota validation"
  type        = bool
  default     = true
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

variable "lifi_api_key" {
  description = "Lifi API key"
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

variable "rate_limiting_ip_whitelist" {
  description = "Comma separated list of whitelisted IPs"
  type        = string
}

#-------------------------------------------------------------------------------
# IRN client configuration

variable "irn_nodes" {
  description = "Comma-separated IRN nodes address in PeerAddr format"
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

variable "internal_api_coinbase_credentials" {
  description = "JWT token for internal API to fetch Coinbase credentials"
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