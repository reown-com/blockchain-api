variable "redis_name" {
  type = string
}

variable "node_type" {
  type = string
}

variable "app_name" {
  type = string
}

variable "allowed_ingress_cidr_blocks" {
  type    = list(string)
  default = null
}

variable "allowed_egress_cidr_blocks" {
  type    = list(string)
  default = null
}

variable "vpc_name" {
  type = string
}

variable "zone_id" {
  type    = string
  default = null
}

variable "zone_name" {
  type    = string
  default = null
}
