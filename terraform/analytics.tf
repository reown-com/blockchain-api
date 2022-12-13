# Analytics S3 Bucket
#tfsec:ignore:aws-s3-enable-bucket-logging
resource "aws_s3_bucket" "analytics_bucket" {
  bucket = "walletconnect.${local.app_name}.${terraform.workspace}.analytics"
}

resource "aws_s3_bucket_acl" "analytics_acl" {
  bucket = aws_s3_bucket.analytics_bucket.id
  acl    = "private"
}

resource "aws_s3_bucket_public_access_block" "analytics_bucket" {
  bucket = aws_s3_bucket.analytics_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_kms_key" "analytics_bucket" {
  description             = "${terraform.workspace} - analytics bucket encryption"
  enable_key_rotation     = true
  deletion_window_in_days = 10
}

resource "aws_kms_alias" "analytics_bucket" {
  target_key_id = aws_kms_key.analytics_bucket.id
  name          = "alias/analytics/proxy/${terraform.workspace}"
}

resource "aws_s3_bucket_server_side_encryption_configuration" "analytics_bucket" {
  bucket = aws_s3_bucket.analytics_bucket.id

  rule {
    apply_server_side_encryption_by_default {
      kms_master_key_id = aws_kms_key.analytics_bucket.arn
      sse_algorithm     = "aws:kms"
    }
  }
}

resource "aws_s3_bucket_versioning" "analytics_bucket" {
  bucket = aws_s3_bucket.analytics_bucket.id

  versioning_configuration {
    status = "Enabled"
  }
}

################################################################################


#tfsec:ignore:aws-s3-enable-bucket-logging
resource "aws_s3_bucket" "analytics-data-lake_bucket" {
  bucket = "walletconnect.${local.app_name}.${terraform.workspace}.analytics.data-lake"
}

resource "aws_s3_bucket_acl" "analytics-data-lake_acl" {
  bucket = aws_s3_bucket.analytics-data-lake_bucket.id
  acl    = "private"
}

resource "aws_s3_bucket_public_access_block" "analytics-data-lake_bucket" {
  bucket = aws_s3_bucket.analytics-data-lake_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "analytics-data-lake_bucket" {
  bucket = aws_s3_bucket.analytics-data-lake_bucket.id

  rule {
    apply_server_side_encryption_by_default {
      kms_master_key_id = aws_kms_key.analytics_bucket.arn
      sse_algorithm     = "aws:kms"
    }
  }
}

resource "aws_s3_bucket_versioning" "analytics-data-lake_bucket" {
  bucket = aws_s3_bucket.analytics-data-lake_bucket.id

  versioning_configuration {
    status = "Enabled"
  }
}
