# Load Balancer
#tfsec:ignore:aws-elb-alb-not-public
resource "aws_alb" "network_load_balancer" {
  name               = replace("${var.app_name}-lb-${substr(uuid(), 0, 3)}", "_", "-")
  load_balancer_type = "network"
  subnets            = var.public_subnets

  lifecycle {
    create_before_destroy = true
    ignore_changes        = [name]
  }
}

resource "aws_lb_target_group" "target_group" {
  name               = replace("${var.app_name}-${substr(uuid(), 0, 3)}", "_", "-")
  port               = var.port
  protocol           = "TCP"
  target_type        = "ip"
  vpc_id             = var.vpc_id
  preserve_client_ip = true

  # Deregister quickly to allow for faster deployments
  deregistration_delay = 30 # Seconds

  health_check {
    protocol            = "HTTP"
    path                = "/health"
    interval            = 10
    healthy_threshold   = 2
    unhealthy_threshold = 2
  }

  lifecycle {
    create_before_destroy = true
    ignore_changes        = [name]
  }
}

resource "aws_lb_listener" "listener" {
  load_balancer_arn = aws_alb.network_load_balancer.arn
  port              = "443"
  protocol          = "TLS"
  certificate_arn   = var.acm_certificate_arn
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.target_group.arn
  }

  lifecycle {
    create_before_destroy = true
  }
}

moved {
  from = aws_security_group.tls_ingess
  to   = aws_security_group.tls_ingress
}

# Security Groups
resource "aws_security_group" "tls_ingress" {
  name        = "${var.app_name}-tls-ingress"
  description = "Allow tls ingress from everywhere"
  vpc_id      = var.vpc_id

  ingress {
    description = "allow TLS traffic from open internet to the proxy"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    #tfsec:ignore:aws-ec2-no-public-ingress-sgr
    cidr_blocks = ["0.0.0.0/0"] # Allowing traffic in from all sources
  }

  egress {           #tfsec:ignore:aws-ec2-add-description-to-security-group-rule
    from_port = 0    # Allowing any incoming port
    to_port   = 0    # Allowing any outgoing port
    protocol  = "-1" # Allowing any outgoing protocol
    #tfsec:ignore:aws-ec2-no-public-egress-sgr
    cidr_blocks = ["0.0.0.0/0"] # Allowing traffic out to all IP addresses
  }
}

resource "aws_security_group" "vpc_app_ingress" {
  name        = "${var.app_name}-vpc-ingress-to-app"
  description = "Allow app port ingress from vpc"
  vpc_id      = var.vpc_id

  ingress {
    description = "allow traffic from open internet to the proxy (needed since lb has client ip forwarding)"
    from_port   = var.port
    to_port     = var.port
    protocol    = "tcp"
    #tfsec:ignore:aws-ec2-no-public-ingress-sgr
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {           #tfsec:ignore:aws-ec2-add-description-to-security-group-rule
    from_port = 0    # Allowing any incoming port
    to_port   = 0    # Allowing any outgoing port
    protocol  = "-1" # Allowing any outgoing protocol
    #tfsec:ignore:aws-ec2-no-public-egress-sgr
    cidr_blocks = ["0.0.0.0/0"] # Allowing traffic out to all IP addresses
  }
}

resource "aws_security_group" "sigv4_proxy_vpc_ingress" {
  name        = "${var.app_name}-vpc-ingress-to-sigv4"
  description = "Allow ingress from inside of vpc to sigv4"
  vpc_id      = var.vpc_id

  ingress {
    description = "allow traffic from inside of cidr block to sigv4 proxy"
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    #tfsec:ignore:aws-ec2-no-public-ingress-sgr
    cidr_blocks = [var.vpc_cidr] # Allowing traffic in from the cidr block 
  }

  egress {           #tfsec:ignore:aws-ec2-add-description-to-security-group-rule
    from_port = 0    # Allowing any incoming port
    to_port   = 0    # Allowing any outgoing port
    protocol  = "-1" # Allowing any outgoing protocol
    #tfsec:ignore:aws-ec2-no-public-egress-sgr
    cidr_blocks = ["0.0.0.0/0"] # Allowing traffic out to all IP addresses
  }
}

# DNS Records
resource "aws_route53_record" "dns_load_balancer" {
  zone_id = var.route53_zone_id
  name    = var.fqdn
  type    = "A"

  alias {
    name                   = aws_alb.network_load_balancer.dns_name
    zone_id                = aws_alb.network_load_balancer.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "backup_dns_load_balancer" {
  zone_id = var.backup_route53_zone_id
  name    = var.backup_fqdn
  type    = "A"

  alias {
    name                   = aws_alb.network_load_balancer.dns_name
    zone_id                = aws_alb.network_load_balancer.zone_id
    evaluate_target_health = true
  }
}

resource "aws_lb_listener_certificate" "backup_cert" {
  listener_arn    = aws_lb_listener.listener.arn
  certificate_arn = var.backup_acm_certificate_arn
}

# VPC Endpoints
# Best practice is to keep traffic VPC internal
# as this is more cost-effective
resource "aws_security_group" "vpc-endpoint-group" {
  name        = "${var.environment}.${var.region}.${var.app_name}-vpc-endpoint"
  description = "Allow tls ingress from VPC"
  vpc_id      = var.vpc_id
  ingress {
    description = "allow TLS traffic from vpc to the proxy"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  egress {
    description = "allow all traffic from the proxy to VPC"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    #tfsec:ignore:aws-ec2-no-public-egress-sgr
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Application = var.app_name
  }
}

resource "aws_vpc_endpoint" "prometheus" {
  vpc_id            = var.vpc_id
  service_name      = "com.amazonaws.${var.region}.aps-workspaces"
  vpc_endpoint_type = "Interface"

  subnet_ids = var.private_subnets

  security_group_ids = [
    aws_security_group.vpc-endpoint-group.id,
  ]

  tags = {
    Application = var.app_name
  }
}

resource "aws_vpc_endpoint" "s3" {
  vpc_id            = var.vpc_id
  service_name      = "com.amazonaws.${var.region}.s3"
  vpc_endpoint_type = "Gateway"

  route_table_ids = var.private_route_table_ids
}

resource "aws_vpc_endpoint" "cloudwatch" {
  vpc_id            = var.vpc_id
  service_name      = "com.amazonaws.${var.region}.logs"
  vpc_endpoint_type = "Interface"

  subnet_ids = var.private_subnets

  security_group_ids = [
    aws_security_group.vpc-endpoint-group.id,
  ]

  tags = {
    Application = var.app_name
  }
}

resource "aws_vpc_endpoint" "ssm" {
  vpc_id            = var.vpc_id
  service_name      = "com.amazonaws.${var.region}.ssm"
  vpc_endpoint_type = "Interface"

  subnet_ids = var.private_subnets

  security_group_ids = [
    aws_security_group.vpc-endpoint-group.id,
  ]

  tags = {
    Application = var.app_name
  }
}

resource "aws_vpc_endpoint" "dkr" {
  vpc_id            = var.vpc_id
  service_name      = "com.amazonaws.${var.region}.ecr.dkr"
  vpc_endpoint_type = "Interface"

  subnet_ids = var.private_subnets

  security_group_ids = [
    aws_security_group.vpc-endpoint-group.id,
  ]

  tags = {
    Application = var.app_name
  }
}

resource "aws_vpc_endpoint" "ecrapi" {
  vpc_id            = var.vpc_id
  service_name      = "com.amazonaws.${var.region}.ecr.api"
  vpc_endpoint_type = "Interface"

  subnet_ids = var.private_subnets

  security_group_ids = [
    aws_security_group.vpc-endpoint-group.id,
  ]

  tags = {
    Application = var.app_name
  }
}
