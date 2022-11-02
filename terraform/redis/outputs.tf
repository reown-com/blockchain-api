output "endpoint" {
  value = var.zone_id == null ? local.cache_endpoint : aws_route53_record.dns[0].fqdn
}

output "cluster_id" {
  value = aws_elasticache_cluster.cache.id
}
