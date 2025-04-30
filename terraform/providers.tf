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

provider "grafana" {
  alias = "sla"
  url   = "https://${data.terraform_remote_state.monitoring.outputs.grafana_workspaces.business.grafana_endpoint}"
  auth  = var.sla_grafana_auth
}
