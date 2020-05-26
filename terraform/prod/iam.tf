resource "aws_iam_role" "dino_park_packs" {
  name = "dino-park-packs-${var.environment}-${var.region}"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "AWS": "${data.terraform_remote_state.kubernetes.outputs.worker_iam_role_arn}"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
EOF
}

resource "aws_iam_role_policy" "dino_park_packs" {
  name   = "dino-park-packs-${var.environment}-${var.region}"
  role   = aws_iam_role.dino_park_packs.name
  policy = data.aws_iam_policy_document.dino_park_packs.json
}

data "aws_iam_policy_document" "dino_park_packs" {
  statement {
    actions = [
      "ses:SendRawEmail",
      "ses:SendEmail"
    ]

    resources = [
      "*"
    ]
  }
}
