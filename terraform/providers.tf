provider "aws" {
  region = var.region

  default_tags {
    tags = module.this.tags
  }
}

provider "grafana" {
  url  = "https://${data.terraform_remote_state.monitoring.outputs.grafana_workspaces.central.grafana_endpoint}"
  auth = var.grafana_auth
}
