output "cluster_id" {
  description = "The ID of the cluster"
  value       = aws_elasticache_cluster.cache.id
}

output "endpoint" {
  description = "The endpoint of the Redis cluster"
  value       = "${aws_elasticache_cluster.cache.cache_nodes[0].address}:${aws_elasticache_cluster.cache.cache_nodes[0].port}"
}
