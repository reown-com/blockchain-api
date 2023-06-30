terraform {
  required_version = "~> 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0.0"
    }
    grafana = {
      source  = "grafana/grafana"
      version = "~> 1.31"
    }
    jsonnet = {
      source  = "alxrem/jsonnet"
      version = "~> 2.2.0"
    }
  }
}

provider "jsonnet" {
  jsonnet_path = "./grafonnet-lib,./panels"
}
