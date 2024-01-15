config {
    format = "default"
    module = true
}

plugin "terraform" {
    enabled = true
    preset  = "all"
}

plugin "aws" {
    enabled = true
    version = "0.18.0"
    source  = "github.com/terraform-linters/tflint-ruleset-aws"
}

rule "terraform_workspace_remote" {
    enabled = false
}
