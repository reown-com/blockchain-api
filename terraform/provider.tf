provider "aws" {
  region = var.region

  # Make it faster by skipping something
  skip_metadata_api_check     = true
  skip_region_validation      = true
  skip_credentials_validation = true
  skip_requesting_account_id  = true

  default_tags {
    tags = module.tags.tags
  }
}

# Expects GRAFANA_AUTH env variable to be set
provider "grafana" {
  url = "https://${var.grafana_endpoint}"
}
