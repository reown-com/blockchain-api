locals {
  vpc_cidr                = "10.0.0.0/16"
  # https://www.notion.so/walletconnect/Private-IP-range-allocation-5fba8350d0a9453ca589dfa73affd508?pvs=4
  # vpc_cidr = (
  #   module.this.stage == "prod" ? {
  #     "eu-central-1"   = "10.0.0.0/16" # "10.7.0.0/16"
  #     "us-east-1"      = "10.8.0.0/16"
  #     "ap-southeast-1" = "10.9.0.0/16"
  #   }
  #   : {
  #     "eu-central-1"   = "10.10.0.0/16"
  #     "us-east-1"      = "10.11.0.0/16"
  #     "ap-southeast-1" = "10.12.0.0/16"
  #   }
  # )[module.this.region]

  vpc_azs                 = slice(data.aws_availability_zones.available.names, 0, 3)
  vpc_flow_s3_bucket_name = substr("vpc-flow-logs-${module.this.id}-${random_pet.this.id}", 0, 63)
}

#-------------------------------------------------------------------------------
# VPC

data "aws_availability_zones" "available" {}

#tfsec:ignore:aws-ec2-no-public-ingress-acl
#tfsec:ignore:aws-ec2-require-vpc-flow-logs-for-all-vpcs
#tfsec:ignore:aws-ec2-no-excessive-port-access
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = module.this.id
  cidr = local.vpc_cidr
  azs  = local.vpc_azs

  database_subnets = [for k, v in local.vpc_azs : cidrsubnet(local.vpc_cidr, 8, k)]
  intra_subnets    = [for k, v in local.vpc_azs : cidrsubnet(local.vpc_cidr, 8, k + 4)]
  public_subnets   = [for k, v in local.vpc_azs : cidrsubnet(local.vpc_cidr, 8, k + 8)]
  private_subnets  = [for k, v in local.vpc_azs : cidrsubnet(local.vpc_cidr, 8, k + 12)]

  enable_dns_support     = true
  enable_dns_hostnames   = true
  enable_nat_gateway     = true
  single_nat_gateway     = true
  one_nat_gateway_per_az = false

  enable_flow_log           = true
  flow_log_file_format      = "parquet"
  flow_log_destination_type = "s3"
  flow_log_destination_arn  = module.vpc_flow_s3_bucket.s3_bucket_arn
  vpc_flow_log_tags         = module.this.tags
}

resource "aws_vpc_peering_connection" "irn" {
  vpc_id        = module.vpc.vpc_id
  peer_vpc_id   = var.irn_vpc_id
  peer_owner_id = var.irn_aws_account_id
}

resource "aws_route" "irn" {
  count = length(module.vpc.private_route_table_ids)

  route_table_id            = module.vpc.private_route_table_ids[count.index]
  vpc_peering_connection_id = aws_vpc_peering_connection.irn.id
  destination_cidr_block    = var.irn_vpc_cidr
}

module "vpc_endpoints" {
  source  = "terraform-aws-modules/vpc/aws//modules/vpc-endpoints"
  version = "5.1"

  vpc_id = module.vpc.vpc_id

  endpoints = {
    cloudwatch = {
      service = "monitoring"
    },
    cloudwatch-events = {
      service = "events"
    },
    cloudwatch-logs = {
      service = "logs"
    },
    ecs = {
      service = "ecs"
    },
    ecs-agent = {
      service = "ecs-agent"
    },
    ecs-telemetry = {
      service = "ecs-telemetry"
    },
    elastic-load-balancing = {
      service = "elasticloadbalancing"
    },
    kms = {
      service = "kms"
    },
    s3 = {
      service = "s3"
    },
  }
}

#-------------------------------------------------------------------------------
# VPC Flow S3 Bucket

#TODO: Enable bucket logging and send logs to bucket on security account.
#tfsec:ignore:aws-s3-enable-versioning
#tfsec:ignore:aws-s3-enable-bucket-logging
#tfsec:ignore:aws-s3-enable-bucket-encryption
#tfsec:ignore:aws-s3-encryption-customer-key
module "vpc_flow_s3_bucket" {
  source  = "terraform-aws-modules/s3-bucket/aws"
  version = "~> 3.14"

  bucket        = local.vpc_flow_s3_bucket_name
  force_destroy = true

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AWSLogDeliveryAclCheck"
        Effect = "Allow"
        Principal = {
          Service = "delivery.logs.amazonaws.com"
        }
        Action   = "s3:GetBucketAcl"
        Resource = "arn:aws:s3:::${local.vpc_flow_s3_bucket_name}"
      },
      {
        Sid    = "AWSLogDeliveryWrite"
        Effect = "Allow"
        Principal = {
          Service = "delivery.logs.amazonaws.com"
        }
        Action   = "s3:PutObject"
        Resource = "arn:aws:s3:::${local.vpc_flow_s3_bucket_name}/AWSLogs/*"
      }
    ]
  })

  lifecycle_rule = [
    {
      id      = "transition-old-logs"
      enabled = true

      transition = [
        {
          days          = 30
          storage_class = "ONEZONE_IA"
        },
        {
          days          = 60
          storage_class = "GLACIER"
        }
      ]
    }
  ]
}
