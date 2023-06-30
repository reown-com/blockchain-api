terraform {
  required_version = "~> 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0.0"
    }
  }
}

data "aws_vpc" "vpc" {
  filter {
    name   = "tag:Name"
    values = [var.vpc_name]
  }
}


# Private Zone
resource "aws_route53_zone" "private_zone" {
  name = var.name

  vpc {
    vpc_id = data.aws_vpc.vpc.id
  }

  lifecycle {
    ignore_changes = [vpc]
  }
}
