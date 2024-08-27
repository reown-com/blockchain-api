output "database_url" {
  description = "The URL used to connect to the cluster"
  value       = module.postgres[0].database_url
}

output "database_vpc_id" {
  description = "ID of the database VPC"
  value       = module.vpc.vpc_id
}

output "database_vpc_cidr" {
  description = "CIDR block of the database VPC"
  value       = module.vpc.intra_subnets_cidr_blocks
}

output "database_client_vpc_peering_connection" {
  description = "Peering connection of database client VPCs"
  value       = aws_vpc_peering_connection.database[0].id
}
