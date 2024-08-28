moved {
  from = data.aws_availability_zones.available
  to   = module.eu_central_1.data.aws_availability_zones.available
}

moved {
  from = data.aws_caller_identity.this
  to   = module.eu_central_1.data.aws_caller_identity.this
}

moved {
  from = data.aws_s3_bucket.geoip
  to   = module.eu_central_1.data.aws_s3_bucket.geoip
}

moved {
  from = data.terraform_remote_state.datalake
  to   = module.eu_central_1.data.terraform_remote_state.datalake
}

moved {
  from = data.terraform_remote_state.infra_aws
  to   = module.eu_central_1.data.terraform_remote_state.infra_aws
}

# moved {
#   from = data.terraform_remote_state.monitoring
#   to   = module.eu_central_1.data.terraform_remote_state.monitoring
# }

moved {
  from = data.terraform_remote_state.org
  to   = module.eu_central_1.data.terraform_remote_state.org
}

moved {
  from = aws_iam_role.application_role
  to   = module.eu_central_1.aws_iam_role.application_role
}

moved {
  from = aws_kms_alias.cloudwatch_logs
  to   = module.eu_central_1.aws_kms_alias.cloudwatch_logs
}

moved {
  from = aws_kms_key.cloudwatch_logs
  to   = module.eu_central_1.aws_kms_key.cloudwatch_logs
}

moved {
  from = aws_prometheus_workspace.prometheus
  to   = module.eu_central_1.aws_prometheus_workspace.prometheus
}

moved {
  from = aws_route.irn[0]
  to   = module.eu_central_1.aws_route.irn[0]
}

moved {
  from = aws_vpc_peering_connection.irn
  to   = module.eu_central_1.aws_vpc_peering_connection.irn
}

moved {
  from = random_pet.this
  to   = module.eu_central_1.random_pet.this
}

moved {
  from = module.alerting.aws_cloudwatch_metric_alarm.ecs_cpu_utilization
  to   = module.eu_central_1.module.alerting.aws_cloudwatch_metric_alarm.ecs_cpu_utilization
}

moved {
  from = module.alerting.aws_cloudwatch_metric_alarm.ecs_mem_utilization
  to   = module.eu_central_1.module.alerting.aws_cloudwatch_metric_alarm.ecs_mem_utilization
}

moved {
  from = module.alerting.aws_cloudwatch_metric_alarm.redis_available_memory
  to   = module.eu_central_1.module.alerting.aws_cloudwatch_metric_alarm.redis_available_memory
}

moved {
  from = module.alerting.aws_cloudwatch_metric_alarm.redis_cpu_utilization
  to   = module.eu_central_1.module.alerting.aws_cloudwatch_metric_alarm.redis_cpu_utilization
}

moved {
  from = module.alerting.aws_sns_topic.cloudwatch_webhook
  to   = module.eu_central_1.module.alerting.aws_sns_topic.cloudwatch_webhook
}

moved {
  from = module.alerting.aws_sns_topic.prometheus_webhook
  to   = module.eu_central_1.module.alerting.aws_sns_topic.prometheus_webhook
}

moved {
  from = module.dns_certificate["Z03351311B68L9FUYSBYK"].data.aws_default_tags.provider
  to   = module.eu_central_1.module.dns_certificate["Z03351311B68L9FUYSBYK"].data.aws_default_tags.provider
}

moved {
  from = module.dns_certificate["Z03351311B68L9FUYSBYK"].data.aws_route53_zone.hosted_zone
  to   = module.eu_central_1.module.dns_certificate["Z03351311B68L9FUYSBYK"].data.aws_route53_zone.hosted_zone
}

moved {
  from = module.dns_certificate["Z03351311B68L9FUYSBYK"].aws_acm_certificate.domain_certificate
  to   = module.eu_central_1.module.dns_certificate["Z03351311B68L9FUYSBYK"].aws_acm_certificate.domain_certificate
}

moved {
  from = module.dns_certificate["Z03351311B68L9FUYSBYK"].aws_route53_record.cert_verification[0]
  to   = module.eu_central_1.module.dns_certificate["Z03351311B68L9FUYSBYK"].aws_route53_record.cert_verification[0]
}

moved {
  from = module.dns_certificate["Z08482453O4S3RHV9217F"].data.aws_default_tags.provider
  to   = module.eu_central_1.module.dns_certificate["Z08482453O4S3RHV9217F"].data.aws_default_tags.provider
}

moved {
  from = module.dns_certificate["Z08482453O4S3RHV9217F"].data.aws_route53_zone.hosted_zone
  to   = module.eu_central_1.module.dns_certificate["Z08482453O4S3RHV9217F"].data.aws_route53_zone.hosted_zone
}

moved {
  from = module.dns_certificate["Z08482453O4S3RHV9217F"].aws_acm_certificate.domain_certificate
  to   = module.eu_central_1.module.dns_certificate["Z08482453O4S3RHV9217F"].aws_acm_certificate.domain_certificate
}

moved {
  from = module.dns_certificate["Z08482453O4S3RHV9217F"].aws_route53_record.cert_verification[0]
  to   = module.eu_central_1.module.dns_certificate["Z08482453O4S3RHV9217F"].aws_route53_record.cert_verification[0]
}

moved {
  from = module.ecs.data.aws_iam_role.ecs_task_execution_role
  to   = module.eu_central_1.module.ecs.data.aws_iam_role.ecs_task_execution_role
}

moved {
  from = module.ecs.aws_appautoscaling_policy.ecs_target_cpu
  to   = module.eu_central_1.module.ecs.aws_appautoscaling_policy.ecs_target_cpu
}

moved {
  from = module.ecs.aws_appautoscaling_policy.ecs_target_memory
  to   = module.eu_central_1.module.ecs.aws_appautoscaling_policy.ecs_target_memory
}

moved {
  from = module.ecs.aws_appautoscaling_target.ecs_target
  to   = module.eu_central_1.module.ecs.aws_appautoscaling_target.ecs_target
}

moved {
  from = module.ecs.aws_cloudwatch_log_group.cluster
  to   = module.eu_central_1.module.ecs.aws_cloudwatch_log_group.cluster
}

moved {
  from = module.ecs.aws_cloudwatch_log_group.otel
  to   = module.eu_central_1.module.ecs.aws_cloudwatch_log_group.otel
}

moved {
  from = module.ecs.aws_cloudwatch_log_group.prometheus_proxy
  to   = module.eu_central_1.module.ecs.aws_cloudwatch_log_group.prometheus_proxy
}

moved {
  from = module.ecs.aws_ecs_cluster.app_cluster
  to   = module.eu_central_1.module.ecs.aws_ecs_cluster.app_cluster
}

moved {
  from = module.ecs.aws_ecs_service.app_service
  to   = module.eu_central_1.module.ecs.aws_ecs_service.app_service
}

moved {
  from = module.ecs.aws_ecs_task_definition.app_task
  to   = module.eu_central_1.module.ecs.aws_ecs_task_definition.app_task
}

moved {
  from = module.ecs.aws_iam_policy.datalake_bucket_access
  to   = module.eu_central_1.module.ecs.aws_iam_policy.datalake_bucket_access
}

moved {
  from = module.ecs.aws_iam_policy.geoip_bucket_access
  to   = module.eu_central_1.module.ecs.aws_iam_policy.geoip_bucket_access
}

moved {
  from = module.ecs.aws_iam_policy.otel
  to   = module.eu_central_1.module.ecs.aws_iam_policy.otel
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.cloudwatch_write_policy
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.cloudwatch_write_policy
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.datalake_bucket_access
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.datalake_bucket_access
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.ecs_task_execution_fetch_ghcr_secret_policy
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.ecs_task_execution_fetch_ghcr_secret_policy
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.ecs_task_execution_role_policy
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.ecs_task_execution_role_policy
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.geoip_bucket_access
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.geoip_bucket_access
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.prometheus_read_policy
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.prometheus_read_policy
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.prometheus_write_policy
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.prometheus_write_policy
}

moved {
  from = module.ecs.aws_iam_role_policy_attachment.ssm_read_only_policy
  to   = module.eu_central_1.module.ecs.aws_iam_role_policy_attachment.ssm_read_only_policy
}

moved {
  from = module.ecs.aws_lb.load_balancer
  to   = module.eu_central_1.module.ecs.aws_lb.load_balancer
}

moved {
  from = module.ecs.aws_lb_listener.listener-http
  to   = module.eu_central_1.module.ecs.aws_lb_listener.listener-http
}

moved {
  from = module.ecs.aws_lb_listener.listener-https
  to   = module.eu_central_1.module.ecs.aws_lb_listener.listener-https
}

moved {
  from = module.ecs.aws_lb_listener_certificate.listener-https["Z08482453O4S3RHV9217F"]
  to   = module.eu_central_1.module.ecs.aws_lb_listener_certificate.listener-https["Z08482453O4S3RHV9217F"]
}

moved {
  from = module.ecs.aws_lb_target_group.target_group
  to   = module.eu_central_1.module.ecs.aws_lb_target_group.target_group
}

moved {
  from = module.ecs.aws_route53_record.dns_load_balancer["Z03351311B68L9FUYSBYK"]
  to   = module.eu_central_1.module.ecs.aws_route53_record.dns_load_balancer["Z03351311B68L9FUYSBYK"]
}

moved {
  from = module.ecs.aws_route53_record.dns_load_balancer["Z08482453O4S3RHV9217F"]
  to   = module.eu_central_1.module.ecs.aws_route53_record.dns_load_balancer["Z08482453O4S3RHV9217F"]
}

moved {
  from = module.ecs.aws_security_group.app_ingress
  to   = module.eu_central_1.module.ecs.aws_security_group.app_ingress
}

moved {
  from = module.ecs.aws_security_group.lb_ingress
  to   = module.eu_central_1.module.ecs.aws_security_group.lb_ingress
}

moved {
  from = module.ecs.random_pet.this
  to   = module.eu_central_1.module.ecs.random_pet.this
}

moved {
  from = module.monitoring.data.jsonnet_file.dashboard
  to   = module.eu_central_1.module.monitoring.data.jsonnet_file.dashboard
}

moved {
  from = module.monitoring.grafana_dashboard.main
  to   = module.eu_central_1.module.monitoring.grafana_dashboard.main
}

moved {
  from = module.monitoring.grafana_data_source.cloudwatch
  to   = module.eu_central_1.module.monitoring.grafana_data_source.cloudwatch
}

moved {
  from = module.monitoring.grafana_data_source.prometheus
  to   = module.eu_central_1.module.monitoring.grafana_data_source.prometheus
}

moved {
  from = module.postgres.data.aws_caller_identity.this
  to   = module.eu_central_1.module.postgres.data.aws_caller_identity.this
}

moved {
  from = module.postgres.aws_db_subnet_group.db_subnets
  to   = module.eu_central_1.module.postgres.aws_db_subnet_group.db_subnets
}

moved {
  from = module.postgres.aws_kms_alias.db_master_password
  to   = module.eu_central_1.module.postgres.aws_kms_alias.db_master_password
}

moved {
  from = module.postgres.aws_kms_key.db_master_password
  to   = module.eu_central_1.module.postgres.aws_kms_key.db_master_password
}

moved {
  from = module.postgres.aws_secretsmanager_secret.db_master_password
  to   = module.eu_central_1.module.postgres.aws_secretsmanager_secret.db_master_password
}

moved {
  from = module.postgres.aws_secretsmanager_secret_version.db_master_password
  to   = module.eu_central_1.module.postgres.aws_secretsmanager_secret_version.db_master_password
}

moved {
  from = module.postgres.random_password.db_master_password[0]
  to   = module.eu_central_1.module.postgres.random_password.db_master_password[0]
}

moved {
  from = module.redis.data.aws_vpc.vpc
  to   = module.eu_central_1.module.redis.data.aws_vpc.vpc
}

moved {
  from = module.redis.aws_elasticache_cluster.cache
  to   = module.eu_central_1.module.redis.aws_elasticache_cluster.cache
}

moved {
  from = module.redis.aws_elasticache_subnet_group.private_subnets
  to   = module.eu_central_1.module.redis.aws_elasticache_subnet_group.private_subnets
}

moved {
  from = module.redis.aws_security_group.service_security_group
  to   = module.eu_central_1.module.redis.aws_security_group.service_security_group
}

moved {
  from = module.vpc.data.aws_caller_identity.current[0]
  to   = module.eu_central_1.module.vpc.data.aws_caller_identity.current[0]
}

moved {
  from = module.vpc.data.aws_partition.current[0]
  to   = module.eu_central_1.module.vpc.data.aws_partition.current[0]
}

moved {
  from = module.vpc.data.aws_region.current[0]
  to   = module.eu_central_1.module.vpc.data.aws_region.current[0]
}

moved {
  from = module.vpc.aws_db_subnet_group.database[0]
  to   = module.eu_central_1.module.vpc.aws_db_subnet_group.database[0]
}

moved {
  from = module.vpc.aws_default_network_acl.this[0]
  to   = module.eu_central_1.module.vpc.aws_default_network_acl.this[0]
}

moved {
  from = module.vpc.aws_default_route_table.default[0]
  to   = module.eu_central_1.module.vpc.aws_default_route_table.default[0]
}

moved {
  from = module.vpc.aws_default_security_group.this[0]
  to   = module.eu_central_1.module.vpc.aws_default_security_group.this[0]
}

moved {
  from = module.vpc.aws_eip.nat[0]
  to   = module.eu_central_1.module.vpc.aws_eip.nat[0]
}

moved {
  from = module.vpc.aws_flow_log.this[0]
  to   = module.eu_central_1.module.vpc.aws_flow_log.this[0]
}

moved {
  from = module.vpc.aws_internet_gateway.this[0]
  to   = module.eu_central_1.module.vpc.aws_internet_gateway.this[0]
}

moved {
  from = module.vpc.aws_nat_gateway.this[0]
  to   = module.eu_central_1.module.vpc.aws_nat_gateway.this[0]
}

moved {
  from = module.vpc.aws_route.private_nat_gateway[0]
  to   = module.eu_central_1.module.vpc.aws_route.private_nat_gateway[0]
}

moved {
  from = module.vpc.aws_route.public_internet_gateway[0]
  to   = module.eu_central_1.module.vpc.aws_route.public_internet_gateway[0]
}

moved {
  from = module.vpc.aws_route_table.intra[0]
  to   = module.eu_central_1.module.vpc.aws_route_table.intra[0]
}

moved {
  from = module.vpc.aws_route_table.private[0]
  to   = module.eu_central_1.module.vpc.aws_route_table.private[0]
}

moved {
  from = module.vpc.aws_route_table.public[0]
  to   = module.eu_central_1.module.vpc.aws_route_table.public[0]
}

moved {
  from = module.vpc.aws_route_table_association.database[0]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.database[0]
}

moved {
  from = module.vpc.aws_route_table_association.database[1]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.database[1]
}

moved {
  from = module.vpc.aws_route_table_association.database[2]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.database[2]
}

moved {
  from = module.vpc.aws_route_table_association.intra[0]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.intra[0]
}

moved {
  from = module.vpc.aws_route_table_association.intra[1]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.intra[1]
}

moved {
  from = module.vpc.aws_route_table_association.intra[2]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.intra[2]
}

moved {
  from = module.vpc.aws_route_table_association.private[0]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.private[0]
}

moved {
  from = module.vpc.aws_route_table_association.private[1]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.private[1]
}

moved {
  from = module.vpc.aws_route_table_association.private[2]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.private[2]
}

moved {
  from = module.vpc.aws_route_table_association.public[0]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.public[0]
}

moved {
  from = module.vpc.aws_route_table_association.public[1]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.public[1]
}

moved {
  from = module.vpc.aws_route_table_association.public[2]
  to   = module.eu_central_1.module.vpc.aws_route_table_association.public[2]
}

moved {
  from = module.vpc.aws_subnet.database[0]
  to   = module.eu_central_1.module.vpc.aws_subnet.database[0]
}

moved {
  from = module.vpc.aws_subnet.database[1]
  to   = module.eu_central_1.module.vpc.aws_subnet.database[1]
}

moved {
  from = module.vpc.aws_subnet.database[2]
  to   = module.eu_central_1.module.vpc.aws_subnet.database[2]
}

moved {
  from = module.vpc.aws_subnet.intra[0]
  to   = module.eu_central_1.module.vpc.aws_subnet.intra[0]
}

moved {
  from = module.vpc.aws_subnet.intra[1]
  to   = module.eu_central_1.module.vpc.aws_subnet.intra[1]
}

moved {
  from = module.vpc.aws_subnet.intra[2]
  to   = module.eu_central_1.module.vpc.aws_subnet.intra[2]
}

moved {
  from = module.vpc.aws_subnet.private[0]
  to   = module.eu_central_1.module.vpc.aws_subnet.private[0]
}

moved {
  from = module.vpc.aws_subnet.private[1]
  to   = module.eu_central_1.module.vpc.aws_subnet.private[1]
}

moved {
  from = module.vpc.aws_subnet.private[2]
  to   = module.eu_central_1.module.vpc.aws_subnet.private[2]
}

moved {
  from = module.vpc.aws_subnet.public[0]
  to   = module.eu_central_1.module.vpc.aws_subnet.public[0]
}

moved {
  from = module.vpc.aws_subnet.public[1]
  to   = module.eu_central_1.module.vpc.aws_subnet.public[1]
}

moved {
  from = module.vpc.aws_subnet.public[2]
  to   = module.eu_central_1.module.vpc.aws_subnet.public[2]
}

moved {
  from = module.vpc.aws_vpc.this[0]
  to   = module.eu_central_1.module.vpc.aws_vpc.this[0]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["cloudwatch"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["cloudwatch"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["cloudwatch-events"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["cloudwatch-events"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["cloudwatch-logs"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["cloudwatch-logs"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["ecs"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["ecs"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["ecs-agent"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["ecs-agent"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["ecs-telemetry"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["ecs-telemetry"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["elastic-load-balancing"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["elastic-load-balancing"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["kms"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["kms"]
}

moved {
  from = module.vpc_endpoints.data.aws_vpc_endpoint_service.this["s3"]
  to   = module.eu_central_1.module.vpc_endpoints.data.aws_vpc_endpoint_service.this["s3"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["cloudwatch"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["cloudwatch"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["cloudwatch-events"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["cloudwatch-events"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["cloudwatch-logs"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["cloudwatch-logs"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["ecs"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["ecs"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["ecs-agent"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["ecs-agent"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["ecs-telemetry"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["ecs-telemetry"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["elastic-load-balancing"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["elastic-load-balancing"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["kms"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["kms"]
}

moved {
  from = module.vpc_endpoints.aws_vpc_endpoint.this["s3"]
  to   = module.eu_central_1.module.vpc_endpoints.aws_vpc_endpoint.this["s3"]
}

moved {
  from = module.vpc_flow_s3_bucket.data.aws_caller_identity.current
  to   = module.eu_central_1.module.vpc_flow_s3_bucket.data.aws_caller_identity.current
}

moved {
  from = module.vpc_flow_s3_bucket.data.aws_partition.current
  to   = module.eu_central_1.module.vpc_flow_s3_bucket.data.aws_partition.current
}

moved {
  from = module.vpc_flow_s3_bucket.data.aws_region.current
  to   = module.eu_central_1.module.vpc_flow_s3_bucket.data.aws_region.current
}

moved {
  from = module.vpc_flow_s3_bucket.aws_s3_bucket.this[0]
  to   = module.eu_central_1.module.vpc_flow_s3_bucket.aws_s3_bucket.this[0]
}

moved {
  from = module.vpc_flow_s3_bucket.aws_s3_bucket_lifecycle_configuration.this[0]
  to   = module.eu_central_1.module.vpc_flow_s3_bucket.aws_s3_bucket_lifecycle_configuration.this[0]
}

moved {
  from = module.vpc_flow_s3_bucket.aws_s3_bucket_public_access_block.this[0]
  to   = module.eu_central_1.module.vpc_flow_s3_bucket.aws_s3_bucket_public_access_block.this[0]
}

moved {
  from = module.monitoring.module.monitoring-role.aws_iam_policy.prometheus[0]
  to   = module.eu_central_1.module.monitoring.module.monitoring-role.aws_iam_policy.prometheus[0]
}

moved {
  from = module.monitoring.module.monitoring-role.aws_iam_role.monitoring
  to   = module.eu_central_1.module.monitoring.module.monitoring-role.aws_iam_role.monitoring
}

moved {
  from = module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["athena"]
  to   = module.eu_central_1.module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["athena"]
}

moved {
  from = module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["cloudwatch"]
  to   = module.eu_central_1.module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["cloudwatch"]
}

moved {
  from = module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["prometheus"]
  to   = module.eu_central_1.module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["prometheus"]
}

moved {
  from = module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["xray"]
  to   = module.eu_central_1.module.monitoring.module.monitoring-role.aws_iam_role_policy_attachment.sources["xray"]
}

moved {
  from = module.postgres.module.db_cluster.data.aws_iam_policy_document.monitoring_rds_assume_role[0]
  to   = module.eu_central_1.module.postgres.module.db_cluster.data.aws_iam_policy_document.monitoring_rds_assume_role[0]
}

moved {
  from = module.postgres.module.db_cluster.data.aws_partition.current
  to   = module.eu_central_1.module.postgres.module.db_cluster.data.aws_partition.current
}

moved {
  from = module.postgres.module.db_cluster.aws_iam_role.rds_enhanced_monitoring[0]
  to   = module.eu_central_1.module.postgres.module.db_cluster.aws_iam_role.rds_enhanced_monitoring[0]
}

moved {
  from = module.postgres.module.db_cluster.aws_iam_role_policy_attachment.rds_enhanced_monitoring[0]
  to   = module.eu_central_1.module.postgres.module.db_cluster.aws_iam_role_policy_attachment.rds_enhanced_monitoring[0]
}

moved {
  from = module.postgres.module.db_cluster.aws_rds_cluster.this[0]
  to   = module.eu_central_1.module.postgres.module.db_cluster.aws_rds_cluster.this[0]
}

moved {
  from = module.postgres.module.db_cluster.aws_rds_cluster_instance.this["1"]
  to   = module.eu_central_1.module.postgres.module.db_cluster.aws_rds_cluster_instance.this["1"]
}

moved {
  from = module.postgres.module.db_cluster.aws_security_group.this[0]
  to   = module.eu_central_1.module.postgres.module.db_cluster.aws_security_group.this[0]
}

moved {
  from = module.postgres.module.db_cluster.aws_security_group_rule.this["vpc_ingress"]
  to   = module.eu_central_1.module.postgres.module.db_cluster.aws_security_group_rule.this["vpc_ingress"]
}

moved {
  from = module.eu_central_1.module.ecs.aws_lb_listener_certificate.listener-https["Z08482453O4S3RHV9217F"]
  to   = module.eu_central_1.module.ecs.aws_lb_listener_certificate.listener-https["1"]
}

moved {
  from = module.eu_central_1.module.ecs.aws_lb_listener_certificate.listener-https["Z08394271XY7LL4G2RE1G"]
  to   = module.eu_central_1.module.ecs.aws_lb_listener_certificate.listener-https["1"]
}
