locals {
  db_master_password = var.db_master_password == "" ? random_password.db_master_password[0].result : var.db_master_password
}

resource "random_password" "db_master_password" {
  count   = var.db_master_password == "" ? 1 : 0
  length  = 16
  special = false
}
