output "endpoint" {
  value = local.cache_endpoint
}

output "cluster_id" {
  value = aws_elasticache_cluster.cache.id
}
