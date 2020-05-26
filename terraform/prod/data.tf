data "aws_caller_identity" "current" {
}

data "terraform_remote_state" "vpc" {
  backend = "s3"

  config = {
    bucket = "eks-terraform-shared-state"
    key    = "prod/us-west-2/vpc/terraform.tfstate"
    region = "us-west-2"
  }
}

data "terraform_remote_state" "kubernetes" {
  backend = "s3"

  config = {
    bucket = "it-sre-state-32046420538"
    key    = "prod/us-west-2/terraform.tfstate"
    region = "us-west-2"
  }
}
