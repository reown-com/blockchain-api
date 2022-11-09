variable "region" {
  type    = string
  default = "eu-central-1"
}

variable "infura_project_id" {
  type = string
}

variable "pokt_project_id" {
  type = string
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
