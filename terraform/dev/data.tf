data "aws_caller_identity" "current" {
}

data "terraform_remote_state" "kubernetes" {
  backend = "s3"

  config = {
    bucket = "it-sre-state-32046420538"
    key    = "state/terraform.tfstate"
    region = "us-west-2"
  }
}
