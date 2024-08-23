# Terraform Configuration
terraform {
  required_version = ">= 1.0"

  backend "remote" {
    hostname     = "app.terraform.io"
    organization = "wallet-connect"
    workspaces {
      prefix = "blockchain-"
    }
  }

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.7"
    }
    grafana = {
      source  = "grafana/grafana"
      version = ">= 2.1"
    }
  }
}
