data "terraform_remote_state" "monitoring" {
  backend = "remote"
  config = {
    organization = "wallet-connect"
    workspaces = {
      name = "monitoring"
    }
  }
}
