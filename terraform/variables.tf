variable "region" {
  type    = string
  default = "eu-central-1"
}

variable "ecr_app_version" {
  description = "The tag of the app image to deploy."
  type        = string
  default     = "latest"
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

variable "azs" {
  type    = list(string)
  default = ["eu-central-1a", "eu-central-1b", "eu-central-1c"]
}

variable "grafana_endpoint" {
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

variable "autoscaling_max_instances" {
  type = number
}

variable "autoscaling_min_instances" {
  type = number
}

variable "analytics_data_lake_kms_key_arn" {
  description = "The ARN of KMS encryption key for the data-lake bucket."
  type        = string
}
