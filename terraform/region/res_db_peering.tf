resource "aws_vpc_peering_connection" "database" {
  count = var.database_vpc_id != null && var.database_vpc_region != null ? 1 : 0

  vpc_id      = module.vpc.vpc_id
  peer_vpc_id = var.database_vpc_id
  peer_region = var.database_vpc_region
  # peer_owner_id = var.database_aws_account_id
}

resource "aws_route" "database" {
  count = var.database_vpc_cidr != null ? length(module.vpc.private_route_table_ids) : 0

  route_table_id            = module.vpc.private_route_table_ids[count.index]
  vpc_peering_connection_id = aws_vpc_peering_connection.irn.id
  destination_cidr_block    = var.database_vpc_cidr
}

resource "aws_vpc_peering_connection_accepter" "database_client" {
  for_each = var.database_client_vpc_peering_connections

  vpc_peering_connection_id = each.key
  auto_accept               = true
}

resource "aws_route" "database_client" {
  for_each = flatten(
    [for route in module.vpc.private_route_table_ids :
      [for id, cidr in var.database_client_vpc_peering_connections : {
        route_table_id            = route
        vpc_peering_connection_id = id
        destination_cidr_block    = cidr
  }]])

  route_table_id            = each.value.route_table_id
  vpc_peering_connection_id = each.value.vpc_peering_connection_id
  destination_cidr_block    = each.value.destination_cidr_block
}
