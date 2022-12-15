output "logging_bucket-arn" {
  value = aws_s3_bucket.logging_bucket.arn
}

output "logging_bucket-id" {
  value = aws_s3_bucket.logging_bucket.id
}
