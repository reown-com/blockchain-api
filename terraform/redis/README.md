# `redis` module

This module creates a Redis database.

<!-- BEGIN_TF_DOCS -->

## Requirements

| Name | Version |
|------|---------|
| <a name="requirement_terraform"></a> [terraform](#requirement\_terraform) | ~> 1.0 |
| <a name="requirement_aws"></a> [aws](#requirement\_aws) | ~> 5.7 |
## Providers

| Name | Version |
|------|---------|
| <a name="provider_aws"></a> [aws](#provider\_aws) | ~> 5.7 |
## Modules

| Name | Source | Version |
|------|--------|---------|
| <a name="module_this"></a> [this](#module\_this) | app.terraform.io/wallet-connect/label/null | 0.3.2 |

## Inputs
| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| <a name="input_context"></a> [context](#input\_context) | Single object for setting entire context at once.<br>See description of individual variables for details.<br>Leave string and numeric variables as `null` to use default value.<br>Individual variable settings (non-null) override settings in context object,<br>except for attributes and tags, which are merged. |  <pre lang="json">any</pre> |  <pre lang="json">{<br>  "attributes": [],<br>  "delimiter": null,<br>  "id\_length\_limit": null,<br>  "label\_key\_case": null,<br>  "label\_order": [],<br>  "label\_value\_case": null,<br>  "name": null,<br>  "namespace": null,<br>  "regex\_replace\_chars": null,<br>  "region": null,<br>  "stage": null,<br>  "tags": {}<br>}</pre> |  no |
| <a name="input_egress_cidr_blocks"></a> [egress\_cidr\_blocks](#input\_egress\_cidr\_blocks) | The CIDR blocks to allow egress to, default to VPC only. |  <pre lang="json">set(string)</pre> |  <pre lang="json">null</pre> |  no |
| <a name="input_ingress_cidr_blocks"></a> [ingress\_cidr\_blocks](#input\_ingress\_cidr\_blocks) | The CIDR blocks to allow ingress from, default to VPC only. |  <pre lang="json">set(string)</pre> |  <pre lang="json">null</pre> |  no |
| <a name="input_node_engine_version"></a> [node\_engine\_version](#input\_node\_engine\_version) | The version of Redis to use |  <pre lang="json">string</pre> |  <pre lang="json">"6.x"</pre> |  no |
| <a name="input_node_type"></a> [node\_type](#input\_node\_type) | The instance type to use for the database nodes |  <pre lang="json">string</pre> |  <pre lang="json">"cache.t4g.micro"</pre> |  no |
| <a name="input_num_cache_nodes"></a> [num\_cache\_nodes](#input\_num\_cache\_nodes) | The number of nodes to create in the cluster |  <pre lang="json">number</pre> |  <pre lang="json">1</pre> |  no |
| <a name="input_subnets_ids"></a> [subnets\_ids](#input\_subnets\_ids) | The list of subnet IDs to create the cluster in |  <pre lang="json">set(string)</pre> |  <pre lang="json">n/a</pre> |  yes |
| <a name="input_vpc_id"></a> [vpc\_id](#input\_vpc\_id) | The VPC ID to create the security group in |  <pre lang="json">string</pre> |  <pre lang="json">n/a</pre> |  yes |
## Outputs

| Name | Description |
|------|-------------|
| <a name="output_cluster_id"></a> [cluster\_id](#output\_cluster\_id) | The ID of the cluster |
| <a name="output_endpoint"></a> [endpoint](#output\_endpoint) | The endpoint of the Redis cluster |


<!-- END_TF_DOCS -->
