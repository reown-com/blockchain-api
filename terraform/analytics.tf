# Analytics S3 Bucket
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
