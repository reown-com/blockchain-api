#-------------------------------------------------------------------------------
# Nodes Configuration

variable "node_type" {
  description = "The instance type to use for the database nodes"
  type        = string
  default     = "cache.t4g.micro" # https://aws.amazon.com/elasticache/pricing/?nc=sn&loc=5#On-demand_nodes
}

variable "num_cache_nodes" {
  description = "The number of nodes to create in the cluster"
  type        = number
  default     = 1
}

variable "node_engine_version" {
  description = "The version of Redis to use"
  type        = string
  default     = "6.x"
}

#-------------------------------------------------------------------------------
# Networking

variable "vpc_id" {
  description = "The VPC ID to create the security group in"
  type        = string
}

variable "subnets_ids" {
  description = "The list of subnet IDs to create the cluster in"
  type        = set(string)
}

variable "ingress_cidr_blocks" {
  description = "The CIDR blocks to allow ingress from, default to VPC only."
  type        = set(string)
  default     = null
}

variable "egress_cidr_blocks" {
  description = "The CIDR blocks to allow egress to, default to VPC only."
  type        = set(string)
  default     = null
}
