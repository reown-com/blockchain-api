# `monitoring` module

Configure the Grafana dashboards for the application

<!-- BEGIN_TF_DOCS -->

## Requirements

| Name | Version |
|------|---------|
| <a name="requirement_terraform"></a> [terraform](#requirement\_terraform) | >= 1.0 |
| <a name="requirement_grafana"></a> [grafana](#requirement\_grafana) | ~> 2.0 |
| <a name="requirement_jsonnet"></a> [jsonnet](#requirement\_jsonnet) | ~> 2.2.0 |
## Providers

| Name | Version |
|------|---------|
| <a name="provider_grafana"></a> [grafana](#provider\_grafana) | ~> 2.0 |
| <a name="provider_jsonnet"></a> [jsonnet](#provider\_jsonnet) | ~> 2.2.0 |
## Modules

| Name | Source | Version |
|------|--------|---------|
| <a name="module_monitoring-role"></a> [monitoring-role](#module\_monitoring-role) | app.terraform.io/wallet-connect/monitoring-role/aws | 1.0.2 |
| <a name="module_this"></a> [this](#module\_this) | app.terraform.io/wallet-connect/label/null | 0.3.2 |

## Inputs
| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| <a name="input_context"></a> [context](#input\_context) | Single object for setting entire context at once.<br>See description of individual variables for details.<br>Leave string and numeric variables as `null` to use default value.<br>Individual variable settings (non-null) override settings in context object,<br>except for attributes and tags, which are merged. |  <pre lang="json">any</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_ecs_service_name"></a> [ecs\_service\_name](#input\_ecs\_service\_name) | The name of the ECS service. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_ecs_target_group_arn"></a> [ecs\_target\_group\_arn](#input\_ecs\_target\_group\_arn) | The ARN of the ECS LB target group. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_ecs_task_family"></a> [ecs\_task\_family](#input\_ecs\_task\_family) | The name of the ECS task family. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_load_balancer_arn"></a> [load\_balancer\_arn](#input\_load\_balancer\_arn) | The ARN of the load balancer. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_monitoring_role_arn"></a> [monitoring\_role\_arn](#input\_monitoring\_role\_arn) | The ARN of the monitoring role. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_notification_channels"></a> [notification\_channels](#input\_notification\_channels) | The notification channels to send alerts to |  <pre lang="json">list(any)</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_prometheus_endpoint"></a> [prometheus\_endpoint](#input\_prometheus\_endpoint) | The endpoint for the Prometheus server. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_redis_cluster_id"></a> [redis\_cluster\_id](#input\_redis\_cluster\_id) | The ID of the keystore DocDB cluster. |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
## Outputs

No outputs.


<!-- END_TF_DOCS -->
