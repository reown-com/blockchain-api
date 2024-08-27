output "database_name" {
  description = "The name of the default database in the cluster"
  value       = var.db_name
}

output "master_username" {
  description = "The username for the master DB user"
  value       = var.db_master_username
}

output "master_password_id" {
  description = "The ID of the database master password in Secrets Manager"
  value       = aws_secretsmanager_secret.db_master_password.id
}

output "rds_cluster_arn" {
  description = "The ARN of the cluster"
  value       = module.db_cluster.cluster_arn
}

output "rds_cluster_id" {
  description = "The ID of the cluster"
  value       = module.db_cluster.cluster_id
}

output "rds_cluster_endpoint" {
  description = "The cluster endpoint"
  value       = module.db_cluster.cluster_endpoint
}

output "database_url" {
  description = "The URL used to connect to the cluster"
  value       = "postgres://${module.db_cluster.cluster_master_username}:${module.db_cluster.cluster_master_password}@${module.db_cluster.cluster_endpoint}:${module.db_cluster.cluster_port}/${var.db_name}"
}
