resource "aws_db_instance" "dino_park_packs_db" {
  name                        = "dino-park-packs-db-${var.environment}-${var.region}"
  identifier                  = "dino-park-packs-db-${var.environment}-${var.region}"
  allocated_storage           = 10
  max_allocated_storage       = 100
  storage_type                = "gp2"
  engine                      = "postgres"
  engine_version              = "11"
  instance_class              = "db.t3.micro"
  allow_major_version_upgrade = true
  username                    = "dinopark"
  password                    = "oneTimePassword"
  db_subnet_group_name        = aws_db_subnet_group.dino_park_packs_db.id
  vpc_security_group_ids      = [aws_security_group.dino_park_packs_db.id]
}

resource "aws_db_subnet_group" "dino_park_packs_db" {
  name = "dino-park-packs-db-${var.environment}-${var.region}"
  description = "Subnet for DinoPark test DB"
  subnet_ids  = flatten([data.terraform_remote_state.vpc.outputs.private_subnets])
}

resource "aws_security_group" "dino_park_packs_db" {
  name = "dino-park-packs-db-${var.environment}-${var.region}"
  vpc_id = data.terraform_remote_state.vpc.outputs.vpc_id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "TCP"
    cidr_blocks = ["10.0.0.0/16"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "TCP"
    cidr_blocks = ["0.0.0.0/0"]
  }
}