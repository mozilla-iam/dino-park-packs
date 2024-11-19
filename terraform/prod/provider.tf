#---
# Provider Configuration
#---

provider "aws" {
  region = "us-west-2"
}

terraform {
  required_version = "~> 0.12"
  required_providers {
    aws = {
      version = "~> 2.62.0"
    }
  }

  backend "s3" {
    bucket = "eks-terraform-shared-state"
    key    = "prod/us-west-2/apps/dino-park-packs-prod/terraform.tfstate"
    region = "us-west-2"
  }
}

