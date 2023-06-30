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

variable "private_subnets" {
  type = set(string)
}

variable "vpc_id" {
  type = string
}

variable "vpc_cidr" {
  type = string
}
