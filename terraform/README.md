# Terraform Infrastructure

Get yourself some AWS creds and then init your workspace:

`terraform -chdir=terraform init`

Use the dev workspace:

`terraform -chdir=terraform workspace select dev`

Now you can apply the changes:

`terraform -chdir=terraform apply -var-file="vars/$(terraform -chdir=terraform workspace show).tfvars"`
