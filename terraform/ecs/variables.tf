variable "ecr_repository_url" {
  type = string
}

variable "app_name" {
  type = string
}

variable "region" {
  type = string
}

variable "vpc_name" {
  type = string
}

variable "port" {
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

variable "infura_project_id" {
  type = string
}

variable "prometheus_endpoint" {
  type = string
}
