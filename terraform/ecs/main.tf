terraform {
  required_version = "~> 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0.0"
    }
  }
}

locals {
  cpu    = var.environment != "dev" ? 1024 : 256
  memory = 2 * local.cpu # 2x is minimum for ECS

  REDIS_MAX_CONNECTIONS = "128"
  // TODO: version the RPC image so we can pin it
  # pinned_latest_tag     = sort(setsubtract(data.aws_ecr_image.service_image.image_tags, ["latest"]))[0]
  // TODO: allow caller to pin version
  image_tag = var.ecr_app_version == "latest" ? data.aws_ecr_image.service_image.image_tags[0] : var.ecr_app_version # TODO: var.ecr_app_version == "latest" ? local.pinned_latest_tag : var.ecr_app_version
  image     = "${var.ecr_repository_url}:${local.image_tag}"

  file_descriptor_soft_limit = pow(2, 18)
  file_descriptor_hard_limit = local.file_descriptor_soft_limit * 2
}

data "aws_ecr_image" "service_image" {
  repository_name = "rpc-proxy"
  image_tag       = "latest"
}

# Log Group for our App
#tfsec:ignore:aws-cloudwatch-log-group-customer-key
resource "aws_cloudwatch_log_group" "cluster_logs" {
  name              = "${var.app_name}_logs"
  retention_in_days = 14
  # TODO: Enable CMK encryption of CloudWatch Log Groups:
  #  kms_key_id = aws_kms_key.log_key.arn
}

# ECS Cluster
resource "aws_ecs_cluster" "app_cluster" {
  name = "${var.app_name}_cluster"

  configuration {
    execute_command_configuration {
      logging = "OVERRIDE"

      log_configuration {
        cloud_watch_encryption_enabled = false
        cloud_watch_log_group_name     = aws_cloudwatch_log_group.cluster_logs.name
      }
    }
  }

  # Exposes metrics such as the
  # number of running tasks
  # in CloudWatch
  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

# ECS Task definition
resource "aws_ecs_task_definition" "app_task" {
  family = var.app_name

  container_definitions = jsonencode([
    {
      name : var.app_name,
      environment : [
        { name : "RPC_PROXY_INFURA_PROJECT_ID", value : tostring(var.infura_project_id) },
        { name : "RPC_PROXY_POKT_PROJECT_ID", value : tostring(var.pokt_project_id) },
        { name : "RPC_PROXY_ZERION_API_KEY", value : tostring(var.zerion_api_key) },

        { name : "RPC_PROXY_REGISTRY_API_URL", value : var.registry_api_endpoint },
        { name : "RPC_PROXY_REGISTRY_API_AUTH_TOKEN", value : var.registry_api_auth_token },
        { name : "RPC_PROXY_REGISTRY_PROJECT_DATA_CACHE_TTL", value : tostring(var.project_data_cache_ttl) },

        { name : "RPC_PROXY_STORAGE_REDIS_MAX_CONNECTIONS", value : tostring(local.REDIS_MAX_CONNECTIONS) },
        { name : "RPC_PROXY_STORAGE_PROJECT_DATA_REDIS_ADDR_READ", value : "redis://${var.project_data_redis_endpoint_read}/0" },
        { name : "RPC_PROXY_STORAGE_PROJECT_DATA_REDIS_ADDR_WRITE", value : "redis://${var.project_data_redis_endpoint_write}/0" },
        { name : "RPC_PROXY_STORAGE_IDENTITY_CACHE_REDIS_ADDR_READ", value : "redis://${var.identity_cache_redis_endpoint_read}/1" },
        { name : "RPC_PROXY_STORAGE_IDENTITY_CACHE_REDIS_ADDR_WRITE", value : "redis://${var.identity_cache_redis_endpoint_write}/1" },

        { "name" : "RPC_PROXY_ANALYTICS_EXPORT_BUCKET", "value" : var.analytics_data_lake_bucket_name },
        { "name" : "RPC_PROXY_ANALYTICS_GEOIP_DB_BUCKET", "value" : var.analytics_geoip_db_bucket_name },
        { "name" : "RPC_PROXY_ANALYTICS_GEOIP_DB_KEY", "value" : var.analytics_geoip_db_key },

        { "name" : "SIG_PROXY_URL", "value" : "http://127.0.0.1:8080/workspaces/${var.prometheus_workspace_id}" },
        { "name" : "SIG_PROM_WORKSPACE_HEADER", "value" : "aps-workspaces.${var.region}.amazonaws.com" },
      ],
      image : local.image,
      essential : true,
      portMappings : [
        {
          containerPort : var.port,
          hostPort : var.port
        }
      ],
      memory : local.memory,
      cpu : local.cpu,
      ulimits : [{
        name : "nofile",
        softLimit : local.file_descriptor_soft_limit,
        hardLimit : local.file_descriptor_hard_limit,
      }],
      logConfiguration : {
        logDriver : "awslogs",
        options : {
          "awslogs-group" : aws_cloudwatch_log_group.cluster_logs.name,
          "awslogs-region" : var.region,
          "awslogs-stream-prefix" : "ecs"
        }
      },
      dependsOn : [{
        containerName : "aws-otel-collector",
        condition : "START"
        },
        {
          containerName : "sigv4-prometheus-proxy",
          condition : "START"
      }]
    },
    {
      name : "aws-otel-collector",
      image : "public.ecr.aws/aws-observability/aws-otel-collector:latest",
      environment : [
        { name : "AWS_PROMETHEUS_SCRAPING_ENDPOINT", value : "0.0.0.0:${var.private_port}" },
        { name : "AWS_PROMETHEUS_ENDPOINT", value : "${var.prometheus_endpoint}api/v1/remote_write" }
      ],
      essential : true,
      command : [
        "--config=/etc/ecs/ecs-amp-prometheus.yaml"
      ],
      logConfiguration : {
        logDriver : "awslogs",
        options : {
          "awslogs-create-group" : "True",
          "awslogs-group" : "/ecs/${var.app_name}-ecs-aws-otel-sidecar-collector",
          "awslogs-region" : var.region,
          "awslogs-stream-prefix" : "ecs"
        }
      }
    },
    {
      name : "sigv4-prometheus-proxy",
      image : "public.ecr.aws/aws-observability/aws-sigv4-proxy:latest",
      essential : true,
      portMappings : [
        {
          containerPort : 8080,
          hostPort : 8080,
        }
      ],
      command : [

      ],
      logConfiguration : {
        logDriver : "awslogs",
        options : {
          "awslogs-create-group" : "True",
          "awslogs-group" : "/ecs/${var.app_name}-ecs-sigv4-prometheus-proxy",
          "awslogs-region" : var.region,
          "awslogs-stream-prefix" : "ecs"
        }
      },

    }
  ])

  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  memory                   = local.memory
  cpu                      = local.cpu
  execution_role_arn       = aws_iam_role.ecs_task_execution_role.arn
  task_role_arn            = aws_iam_role.ecs_task_execution_role.arn

  runtime_platform {
    operating_system_family = "LINUX"
  }
}

# ECS Service
resource "aws_ecs_service" "app_service" {
  name            = "${var.app_name}-service"
  cluster         = aws_ecs_cluster.app_cluster.id
  task_definition = join(":", slice(split(":", aws_ecs_task_definition.app_task.arn), 0, 6))
  launch_type     = "FARGATE"
  desired_count   = 2 # Setting the number of containers we want deployed to 3
  propagate_tags  = "TASK_DEFINITION"

  # Wait for the service deployment to succeed
  wait_for_steady_state = true

  network_configuration {
    subnets          = var.private_subnets
    assign_public_ip = false                                                                                                                     # We do public ingress through the LB
    security_groups  = [aws_security_group.tls_ingress.id, aws_security_group.vpc_app_ingress.id, aws_security_group.sigv4_proxy_vpc_ingress.id] # Setting the security group
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.target_group.arn
    container_name   = aws_ecs_task_definition.app_task.family
    container_port   = var.port
  }

  # Allow external changes without Terraform plan difference
  lifecycle {
    ignore_changes = [desired_count]
  }
}

#  Autoscaling
# We can scale by
# ECSServiceAverageCPUUtilization, ECSServiceAverageMemoryUtilization, and ALBRequestCountPerTarget
# out of the box or use custom metrics
resource "aws_appautoscaling_target" "ecs_target" {
  max_capacity       = var.autoscaling_max_capacity
  min_capacity       = var.autoscaling_min_capacity
  resource_id        = "service/${aws_ecs_cluster.app_cluster.name}/${aws_ecs_service.app_service.name}"
  scalable_dimension = "ecs:service:DesiredCount"
  service_namespace  = "ecs"
}

resource "aws_appautoscaling_policy" "cpu_scaling" {
  name               = "${var.app_name}-application-scaling-policy-cpu"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.ecs_target.resource_id
  scalable_dimension = aws_appautoscaling_target.ecs_target.scalable_dimension
  service_namespace  = aws_appautoscaling_target.ecs_target.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "ECSServiceAverageCPUUtilization"
    }
    target_value       = 70
    scale_in_cooldown  = 180
    scale_out_cooldown = 180
  }
  depends_on = [aws_appautoscaling_target.ecs_target]
}

resource "aws_appautoscaling_policy" "memory_scaling" {
  name               = "${var.app_name}-application-scaling-policy-memory"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.ecs_target.resource_id
  scalable_dimension = aws_appautoscaling_target.ecs_target.scalable_dimension
  service_namespace  = aws_appautoscaling_target.ecs_target.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "ECSServiceAverageMemoryUtilization"
    }
    target_value       = 70
    scale_in_cooldown  = 180
    scale_out_cooldown = 180
  }
  depends_on = [aws_appautoscaling_target.ecs_target]
}
