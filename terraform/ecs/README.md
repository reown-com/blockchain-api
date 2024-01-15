# `ecs` module

This module creates an ECS cluster and an autoscaling group of EC2 instances to run the application.

<!-- BEGIN_TF_DOCS -->

## Requirements

| Name | Version |
|------|---------|
| <a name="requirement_terraform"></a> [terraform](#requirement\_terraform) | ~> 1.0 |
| <a name="requirement_aws"></a> [aws](#requirement\_aws) | ~> 5.7 |
| <a name="requirement_random"></a> [random](#requirement\_random) | 3.5.1 |
## Providers

| Name | Version |
|------|---------|
| <a name="provider_aws"></a> [aws](#provider\_aws) | ~> 5.7 |
| <a name="provider_random"></a> [random](#provider\_random) | 3.5.1 |
## Modules

| Name | Source | Version |
|------|--------|---------|
| <a name="module_ecs_cpu_mem"></a> [ecs\_cpu\_mem](#module\_ecs\_cpu\_mem) | app.terraform.io/wallet-connect/ecs_cpu_mem/aws | 1.0.0 |
| <a name="module_this"></a> [this](#module\_this) | app.terraform.io/wallet-connect/label/null | 0.3.2 |

## Inputs
| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| <a name="input_allowed_app_ingress_cidr_blocks"></a> [allowed\_app\_ingress\_cidr\_blocks](#input\_allowed\_app\_ingress\_cidr\_blocks) | A list of CIDR blocks to allow ingress access to the application. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_allowed_lb_ingress_cidr_blocks"></a> [allowed\_lb\_ingress\_cidr\_blocks](#input\_allowed\_lb\_ingress\_cidr\_blocks) | A list of CIDR blocks to allow ingress access to the load-balancer. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_analytics_datalake_bucket_name"></a> [analytics\_datalake\_bucket\_name](#input\_analytics\_datalake\_bucket\_name) | The name of the S3 bucket to use for the analytics datalake |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_analytics_datalake_kms_key_arn"></a> [analytics\_datalake\_kms\_key\_arn](#input\_analytics\_datalake\_kms\_key\_arn) | The ARN of the KMS key to use with the datalake bucket |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_autoscaling_cpu_scale_in_cooldown"></a> [autoscaling\_cpu\_scale\_in\_cooldown](#input\_autoscaling\_cpu\_scale\_in\_cooldown) | The cooldown period (in seconds) before a scale in is possible |  <pre lang="json">number</pre> |  <pre lang="json">180</pre> |  no |
| <a name="input_autoscaling_cpu_scale_out_cooldown"></a> [autoscaling\_cpu\_scale\_out\_cooldown](#input\_autoscaling\_cpu\_scale\_out\_cooldown) | The cooldown period (in seconds) before a scale out is possible |  <pre lang="json">number</pre> |  <pre lang="json">180</pre> |  no |
| <a name="input_autoscaling_cpu_target"></a> [autoscaling\_cpu\_target](#input\_autoscaling\_cpu\_target) | The target CPU utilization for the autoscaling group |  <pre lang="json">number</pre> |  <pre lang="json">50</pre> |  no |
| <a name="input_autoscaling_desired_count"></a> [autoscaling\_desired\_count](#input\_autoscaling\_desired\_count) | Minimum number of instances in the autoscaling group |  <pre lang="json">number</pre> |  <pre lang="json">2</pre> |  no |
| <a name="input_autoscaling_max_capacity"></a> [autoscaling\_max\_capacity](#input\_autoscaling\_max\_capacity) | Maximum number of instances in the autoscaling group |  <pre lang="json">number</pre> |  <pre lang="json">8</pre> |  no |
| <a name="input_autoscaling_memory_scale_in_cooldown"></a> [autoscaling\_memory\_scale\_in\_cooldown](#input\_autoscaling\_memory\_scale\_in\_cooldown) | The cooldown period (in seconds) before a scale in is possible |  <pre lang="json">number</pre> |  <pre lang="json">180</pre> |  no |
| <a name="input_autoscaling_memory_scale_out_cooldown"></a> [autoscaling\_memory\_scale\_out\_cooldown](#input\_autoscaling\_memory\_scale\_out\_cooldown) | The cooldown period (in seconds) before a scale out is possible |  <pre lang="json">number</pre> |  <pre lang="json">180</pre> |  no |
| <a name="input_autoscaling_memory_target"></a> [autoscaling\_memory\_target](#input\_autoscaling\_memory\_target) | The target memory utilization for the autoscaling group |  <pre lang="json">number</pre> |  <pre lang="json">50</pre> |  no |
| <a name="input_autoscaling_min_capacity"></a> [autoscaling\_min\_capacity](#input\_autoscaling\_min\_capacity) | Minimum number of instances in the autoscaling group |  <pre lang="json">number</pre> |  <pre lang="json">2</pre> |  no |
| <a name="input_cloudwatch_logs_key_arn"></a> [cloudwatch\_logs\_key\_arn](#input\_cloudwatch\_logs\_key\_arn) | The ARN of the KMS key to use for encrypting CloudWatch logs |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_cloudwatch_retention_in_days"></a> [cloudwatch\_retention\_in\_days](#input\_cloudwatch\_retention\_in\_days) | The number of days to retain CloudWatch logs for the DB instance |  <pre lang="json">number</pre> |  <pre lang="json">14</pre> |  no |
| <a name="input_context"></a> [context](#input\_context) | Single object for setting entire context at once.<br>See description of individual variables for details.<br>Leave string and numeric variables as `null` to use default value.<br>Individual variable settings (non-null) override settings in context object,<br>except for attributes and tags, which are merged. |  <pre lang="json">any</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_ecr_repository_url"></a> [ecr\_repository\_url](#input\_ecr\_repository\_url) | The URL of the ECR repository where the app image is stored |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_geoip_db_bucket_name"></a> [geoip\_db\_bucket\_name](#input\_geoip\_db\_bucket\_name) | The name of the S3 bucket where the GeoIP database is stored |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_geoip_db_key"></a> [geoip\_db\_key](#input\_geoip\_db\_key) | The key of the GeoIP database in the S3 bucket |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_identity_cache_endpoint_read"></a> [identity\_cache\_endpoint\_read](#input\_identity\_cache\_endpoint\_read) | The endpoint of the identity cache (read) |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_identity_cache_endpoint_write"></a> [identity\_cache\_endpoint\_write](#input\_identity\_cache\_endpoint\_write) | The endpoint of the identity cache (write) |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_image_version"></a> [image\_version](#input\_image\_version) | The version of the app image to deploy |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_infura_project_id"></a> [infura\_project\_id](#input\_infura\_project\_id) | The project ID for Infura |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_log_level"></a> [log\_level](#input\_log\_level) | The log level for the app |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_ofac_blocked_countries"></a> [ofac\_blocked\_countries](#input\_ofac\_blocked\_countries) | The list of countries under OFAC sanctions |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_pokt_project_id"></a> [pokt\_project\_id](#input\_pokt\_project\_id) | The project ID for POKT |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_port"></a> [port](#input\_port) | The port the app listens on |  <pre lang="json">number</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_private_subnets"></a> [private\_subnets](#input\_private\_subnets) | The IDs of the private subnets |  <pre lang="json">list(string)</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_project_cache_endpoint_read"></a> [project\_cache\_endpoint\_read](#input\_project\_cache\_endpoint\_read) | The endpoint of the project cache (read) |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_project_cache_endpoint_write"></a> [project\_cache\_endpoint\_write](#input\_project\_cache\_endpoint\_write) | The endpoint of the project cache (write) |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_project_cache_ttl"></a> [project\_cache\_ttl](#input\_project\_cache\_ttl) | The TTL for project data cache |  <pre lang="json">number</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_prometheus_endpoint"></a> [prometheus\_endpoint](#input\_prometheus\_endpoint) | The endpoint of the Prometheus server to use for monitoring |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_prometheus_workspace_id"></a> [prometheus\_workspace\_id](#input\_prometheus\_workspace\_id) | The workspace ID of the Prometheus server used for monitoring |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_public_subnets"></a> [public\_subnets](#input\_public\_subnets) | The IDs of the public subnets |  <pre lang="json">list(string)</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_redis_max_connections"></a> [redis\_max\_connections](#input\_redis\_max\_connections) | The maximum number of connections to the Redis server |  <pre lang="json">number</pre> |  <pre lang="json">128</pre> |  no |
| <a name="input_registry_api_auth_token"></a> [registry\_api\_auth\_token](#input\_registry\_api\_auth\_token) | The auth token for the registry API |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_registry_api_endpoint"></a> [registry\_api\_endpoint](#input\_registry\_api\_endpoint) | The endpoint of the registry API |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_route53_zones"></a> [route53\_zones](#input\_route53\_zones) | The FQDNs to use for the app |  <pre lang="json">map(string)</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_route53_zones_certificates"></a> [route53\_zones\_certificates](#input\_route53\_zones\_certificates) | The ARNs of the ACM certificates to use for HTTPS |  <pre lang="json">map(string)</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_task_cpu"></a> [task\_cpu](#input\_task\_cpu) | The number of CPU units to reserve for the container. |  <pre lang="json">number</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_task_execution_role_name"></a> [task\_execution\_role\_name](#input\_task\_execution\_role\_name) | The name of the task execution role |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_task_memory"></a> [task\_memory](#input\_task\_memory) | The amount of memory (in MiB) to reserve for the container. |  <pre lang="json">number</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_vpc_id"></a> [vpc\_id](#input\_vpc\_id) | The ID of the VPC to deploy to |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_zerion_api_key"></a> [zerion\_api\_key](#input\_zerion\_api\_key) | The API key for Zerion |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
## Outputs

| Name | Description |
|------|-------------|
| <a name="output_ecs_cluster_name"></a> [ecs\_cluster\_name](#output\_ecs\_cluster\_name) | The name of the ECS cluster |
| <a name="output_ecs_service_name"></a> [ecs\_service\_name](#output\_ecs\_service\_name) | The name of the ECS service |
| <a name="output_ecs_task_family"></a> [ecs\_task\_family](#output\_ecs\_task\_family) | The family of the task definition |
| <a name="output_load_balancer_arn"></a> [load\_balancer\_arn](#output\_load\_balancer\_arn) | The ARN of the load balancer |
| <a name="output_load_balancer_arn_suffix"></a> [load\_balancer\_arn\_suffix](#output\_load\_balancer\_arn\_suffix) | The ARN suffix of the load balancer |
| <a name="output_service_security_group_id"></a> [service\_security\_group\_id](#output\_service\_security\_group\_id) | The ID of the security group for the service |
| <a name="output_target_group_arn"></a> [target\_group\_arn](#output\_target\_group\_arn) | The ARN of the target group |


<!-- END_TF_DOCS -->
