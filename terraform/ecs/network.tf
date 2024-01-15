locals {
  lb_name = trimsuffix(substr(replace("${module.this.id}-${random_pet.this.id}", "_", "-"), 0, 32), "-")
}

#tfsec:ignore:aws-elb-drop-invalid-headers
#tfsec:ignore:aws-elb-alb-not-public
resource "aws_lb" "load_balancer" {
  name               = local.lb_name
  load_balancer_type = "application"
  subnets            = var.public_subnets

  security_groups = [aws_security_group.lb_ingress.id]

  lifecycle {
    create_before_destroy = true
  }
}

locals {
  main_certificate_key    = keys(var.route53_zones_certificates)[0]
  main_certificate        = var.route53_zones_certificates[local.main_certificate_key]
  additional_certificates = { for k, v in var.route53_zones_certificates : k => v if k != local.main_certificate_key }
}

resource "aws_lb_listener" "listener-https" {
  load_balancer_arn = aws_lb.load_balancer.arn
  port              = "443"
  protocol          = "HTTPS"
  certificate_arn   = local.main_certificate
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.target_group.arn
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_lb_listener_certificate" "listener-https" {
  for_each        = local.additional_certificates
  listener_arn    = aws_lb_listener.listener-https.arn
  certificate_arn = each.value
}

resource "aws_lb_listener" "listener-http" {
  load_balancer_arn = aws_lb.load_balancer.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type = "redirect"

    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_lb_target_group" "target_group" {
  name        = local.lb_name
  port        = var.port
  protocol    = "HTTP"
  target_type = "ip"
  vpc_id      = var.vpc_id
  slow_start  = 30

  health_check {
    protocol            = "HTTP"
    path                = "/health" # Blockchain-API health path
    port                = var.port
    interval            = 15
    timeout             = 10
    healthy_threshold   = 2
    unhealthy_threshold = 2
  }

  lifecycle {
    create_before_destroy = true
  }
}

# Security Groups

#tfsec:ignore:aws-ec2-no-public-ingress-sgr
resource "aws_security_group" "lb_ingress" {
  name        = "${local.lb_name}-lb-ingress"
  description = "Allow app port ingress from vpc"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "Allow HTTPS traffic from anywhere"
  }

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "Allow HTTP traffic from anywhere"
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = [var.allowed_lb_ingress_cidr_blocks]
    description = "Allow traffic out to all VPC IP addresses"
  }

  lifecycle {
    create_before_destroy = true
  }
}

#tfsec:ignore:aws-ec2-no-public-egress-sgr
resource "aws_security_group" "app_ingress" {
  name        = "${local.lb_name}-app-ingress"
  description = "Allow app port ingress"
  vpc_id      = var.vpc_id

  ingress {
    from_port       = 0
    to_port         = 0
    protocol        = "-1"
    security_groups = [aws_security_group.lb_ingress.id]
    description     = "Allow traffic from load balancer"
  }

  ingress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = [var.allowed_app_ingress_cidr_blocks]
    description = "Allow traffic from allowed CIDR blocks"
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
    description = "Allow traffic out to all IP addresses"
  }

  lifecycle {
    create_before_destroy = true
  }
}
