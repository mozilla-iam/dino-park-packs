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

resource "aws_iam_role_policy" "dino_park_packs_ssm_access" {
  name = "dino-park-packs-ssm-access-${var.environment}-${var.region}"
  role = aws_iam_role.dino_park_packs.id

  policy = <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Action": [
                "ssm:GetParameterHistory",
                "ssm:GetParametersByPath",
                "ssm:GetParameters",
                "ssm:GetParameter"
            ],
            "Resource": [
                "arn:aws:ssm:us-west-2:${data.aws_caller_identity.current.account_id}:parameter/iam/cis/production/*"
            ],
            "Effect": "Allow"
        },
        {
            "Action": [
                "kms:Decrypt"
            ],
            "Resource": [
                "arn:aws:kms:us-west-2:320464205386:key/ef00015d-739b-456d-a92f-482712af4f32"
            ],
            "Effect": "Allow"
        }
    ]
}
EOF

}
