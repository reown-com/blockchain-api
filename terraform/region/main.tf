data "aws_caller_identity" "this" {}

resource "random_pet" "this" {
  length = 2
}
