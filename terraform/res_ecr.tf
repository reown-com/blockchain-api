data "aws_ecr_repository" "blockchain" {
  name = "blockchain"
}

resource "aws_ecr_lifecycle_policy" "blockchain_keep_30" {
  repository = data.aws_ecr_repository.blockchain.name

  policy = jsonencode({
    rules = [
      {
        rulePriority = 1
        description  = "Expire images beyond the last 30"
        selection = {
          tagStatus   = "any"
          countType   = "imageCountMoreThan"
          countNumber = 30
        }
        action = {
          type = "expire"
        }
      }
    ]
  })
}
