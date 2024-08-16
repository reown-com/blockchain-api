provider "aws" {
  alias  = "eu-central-1"
  region = var.region

  default_tags {
    tags = module.this.tags
  }
}

provider "aws" {
  alias  = "us-east-1"
  region = "us-east-1"

  default_tags {
    tags = module.this.tags
  }
}

provider "aws" {
  alias  = "ap-southeast-1"
  region = "ap-southeast-1"

  default_tags {
    tags = module.this.tags
  }
}

provider "grafana" {
  url  = "https://${data.terraform_remote_state.monitoring.outputs.grafana_workspaces.central.grafana_endpoint}"
  auth = var.grafana_auth
}
