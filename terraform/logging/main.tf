#tfsec:ignore:aws-s3-enable-bucket-logging
resource "aws_s3_bucket" "logging_bucket" {
  bucket = "walletconnect.${var.app_name}.${terraform.workspace}.access-logs"
}

# https://github.com/hashicorp/terraform-provider-aws/issues/28353
resource "aws_s3_bucket_ownership_controls" "logging_bucket-controls" {
  bucket = aws_s3_bucket.logging_bucket.id
  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_acl" "logging_bucket-acl" {
  depends_on = [aws_s3_bucket_ownership_controls.logging_bucket-controls]

  bucket = aws_s3_bucket.logging_bucket.id
  acl    = "log-delivery-write"
}

resource "aws_s3_bucket_public_access_block" "logging_bucket" {
  bucket = aws_s3_bucket.logging_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

#tfsec:ignore:aws-s3-encryption-customer-key
resource "aws_s3_bucket_server_side_encryption_configuration" "logging_bucket" {
  bucket = aws_s3_bucket.logging_bucket.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
    bucket_key_enabled = true
  }
}

resource "aws_s3_bucket_versioning" "logging_bucket" {
  bucket = aws_s3_bucket.logging_bucket.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "logging_bucket" {
  bucket = aws_s3_bucket.logging_bucket.id

  rule {
    id     = "delete_logs_older_than_7_days"
    status = "Enabled"
    filter {
      prefix = "logs/"
    }
    expiration {
      days = 7
    }
  }
}

data "aws_iam_policy_document" "allow_logging_service" {
  statement {
    sid = "S3ServerAccessLogsPolicy"

    principals {
      type        = "Service"
      identifiers = ["logging.s3.amazonaws.com"]
    }

    actions = [
      "s3:PutObject"
    ]

    resources = [
      aws_s3_bucket.logging_bucket.arn,
      "${aws_s3_bucket.logging_bucket.arn}/*",
    ]
  }
}

resource "aws_s3_bucket_policy" "allow_logging_service" {
  bucket = aws_s3_bucket.logging_bucket.id
  policy = data.aws_iam_policy_document.allow_logging_service.json
}
