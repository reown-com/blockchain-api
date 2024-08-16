#-------------------------------------------------------------------------------
# Database configuration

variable "db_name" {
  description = "The name of the default database in the cluster"
  type        = string
  default     = "postgres"
}

variable "db_master_username" {
  description = "The username for the master DB user"
  type        = string
  default     = "pgadmin"
}

variable "db_master_password" {
  description = "The password for the master DB user"
  type        = string
  default     = ""
}

#-------------------------------------------------------------------------------
# Capacity

variable "instances" {
  description = "The number of database instances to create"
  type        = number
  default     = 1
}

variable "min_capacity" {
  description = "The minimum capacity for the Aurora cluster (in Aurora Capacity Units)"
  type        = number
  default     = 2
}

variable "max_capacity" {
  description = "The maximum capacity for the Aurora cluster (in Aurora Capacity Units)"
  type        = number
  default     = 10
}

#-------------------------------------------------------------------------------
# Logs

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
# Networking

variable "vpc_id" {
  description = "The VPC ID to create the security group in"
  type        = string
}

variable "subnet_ids" {
  description = "The IDs of the subnets to deploy to"
  type        = list(string)
}

variable "ingress_cidr_blocks" {
  description = "The CIDR blocks to allow ingress from"
  type        = list(string)
}
