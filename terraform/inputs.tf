data "terraform_remote_state" "org" {
  backend = "remote"
  config = {
    organization = "wallet-connect"
    workspaces = {
      name = "aws-org"
    }
  }
}

data "terraform_remote_state" "datalake" {
  backend = "remote"
  config = {
    organization = "wallet-connect"
    workspaces = {
      name = "datalake-${module.stage.dev ? "staging" : local.stage}"
    }
  }
}

data "terraform_remote_state" "infra_aws" {
  backend = "remote"
  config = {
    organization = "wallet-connect"
    workspaces = {
      name = "infra-aws"
    }
  }
}

data "terraform_remote_state" "monitoring" {
  backend = "remote"
  config = {
    organization = "wallet-connect"
    workspaces = {
      name = "monitoring"
    }
  }
}
