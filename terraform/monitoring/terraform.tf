terraform {
  required_version = ">= 1.0"

  required_providers {
    grafana = {
      source                = "grafana/grafana"
      version               = ">= 2.1"
      configuration_aliases = [grafana.sla]
    }
    jsonnet = {
      source  = "alxrem/jsonnet"
      version = "~> 2.2.0"
    }
  }
}
