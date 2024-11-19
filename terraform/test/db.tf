resource "aws_db_instance" "dino_park_packs_db" {
  identifier                          = "dino-park-packs-db-${var.environment}-${var.region}"
  allocated_storage                   = 10
  max_allocated_storage               = 100
  storage_type                        = "gp2"
  engine                              = "postgres"
  engine_version                      = "11"
  instance_class                      = "db.t3.micro"
  allow_major_version_upgrade         = true
  username                            = "dinopark"
  password                            = "oneTimePassword"
  db_subnet_group_name                = aws_db_subnet_group.dino_park_packs_db.id
  vpc_security_group_ids              = [aws_security_group.dino_park_packs_db.id]
  iam_database_authentication_enabled = true
  # Saturdays, at 3:00 AM (UTC); 7:00 PM (PST); 10:00 PM (EST) to
  #               5:00 AM (UTC); 9:00 PM (PST); 12:00 AM (EST), respectively.
  maintenance_window = "Sat:03:00-Sat:05:00"
  # Backup every day at 2:00 AM (UTC); 6:00 PM (PST); 9:00 PM (EST) to
  #                     2:59 AM (UTC); 6:69 PM (PST); 9:59 PM (EST), respectively.
  backup_window           = "02:00-02:59"
  backup_retention_period = "15" # days
  copy_tags_to_snapshot   = true
}

resource "aws_db_subnet_group" "dino_park_packs_db" {
  name        = "dino-park-packs-db-${var.environment}-${var.region}"
  description = "Subnet for DinoPark test DB"
  subnet_ids  = flatten([data.terraform_remote_state.vpc.outputs.private_subnets])
}

resource "aws_security_group" "dino_park_packs_db" {
  name   = "dino-park-packs-db-${var.environment}-${var.region}"
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
