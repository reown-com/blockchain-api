locals {
  image = "${var.ecr_repository_url}:${var.image_version}"

  desired_count = module.this.stage == "prod" ? var.autoscaling_desired_count : 1

  task_cpu    = module.this.stage == "prod" ? var.task_cpu : 256
  task_memory = module.this.stage == "prod" ? var.task_memory : 512

  otel_port   = var.port + 1
  otel_cpu    = module.this.stage == "prod" ? 128 : 64
  otel_memory = module.this.stage == "prod" ? 128 : 64

  prometheus_proxy_port   = var.port + 2
  prometheus_proxy_cpu    = module.this.stage == "prod" ? 128 : 64
  prometheus_proxy_memory = module.this.stage == "prod" ? 128 : 64

  file_descriptor_soft_limit = pow(2, 18)
  file_descriptor_hard_limit = local.file_descriptor_soft_limit * 2
}

module "ecs_cpu_mem" {
  source  = "app.terraform.io/wallet-connect/ecs_cpu_mem/aws"
  version = "1.0.0"
  cpu     = local.task_cpu
  memory  = local.task_memory
}

#-------------------------------------------------------------------------------
# ECS Cluster

resource "aws_ecs_cluster" "app_cluster" {
  name = "${module.this.id}-cluster"

  configuration {
    execute_command_configuration {
      logging = "OVERRIDE"

      log_configuration {
        cloud_watch_encryption_enabled = false
        cloud_watch_log_group_name     = aws_cloudwatch_log_group.cluster.name
      }
    }
  }

  # Exposes metrics such as the number of running tasks in CloudWatch
  # Should be disabled because we use Prometheus for CPU and Memory monitoring
  setting {
    name  = "containerInsights"
    value = "disabled"
  }
}

#-------------------------------------------------------------------------------
# ECS Task definition

resource "aws_ecs_task_definition" "app_task" {
  family = module.this.id

  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc" # Using awsvpc as our network mode as this is required for Fargate
  cpu                      = module.ecs_cpu_mem.cpu
  memory                   = module.ecs_cpu_mem.memory
  execution_role_arn       = data.aws_iam_role.ecs_task_execution_role.arn
  task_role_arn            = data.aws_iam_role.ecs_task_execution_role.arn

  runtime_platform {
    operating_system_family = "LINUX"
  }

  container_definitions = jsonencode([
    {
      name      = module.this.id,
      image     = local.image,
      cpu       = local.task_cpu - local.otel_cpu - local.prometheus_proxy_cpu,
      memory    = local.task_memory - local.otel_memory - local.prometheus_proxy_memory,
      essential = true,

      environment = [
        { name = "RPC_PROXY_PORT", value = tostring(var.port) },
        { name = "RPC_PROXY_PROMETHEUS_PORT", value = tostring(local.otel_port) },

        { name = "RPC_PROXY_GEOIP_DB_BUCKET", value = var.geoip_db_bucket_name },
        { name = "RPC_PROXY_GEOIP_DB_KEY", value = var.geoip_db_key },
        { name = "RPC_PROXY_TESTING_PROJECT_ID", value = var.testing_project_id },

        { name = "RPC_PROXY_BLOCKED_COUNTRIES", value = var.ofac_blocked_countries },

        { name = "RPC_PROXY_PROVIDER_INFURA_PROJECT_ID", value = var.infura_project_id },
        { name = "RPC_PROXY_PROVIDER_POKT_PROJECT_ID", value = var.pokt_project_id },
        { name = "RPC_PROXY_PROVIDER_QUICKNODE_API_TOKEN", value = var.quicknode_api_token },
        { name = "RPC_PROXY_PROVIDER_ZERION_API_KEY", value = var.zerion_api_key },
        { name = "RPC_PROXY_PROVIDER_COINBASE_API_KEY", value = var.coinbase_api_key },
        { name = "RPC_PROXY_PROVIDER_COINBASE_APP_ID", value = var.coinbase_app_id },
        { name = "RPC_PROXY_PROVIDER_ONE_INCH_API_KEY", value = var.one_inch_api_key },
        { name = "RPC_PROXY_PROVIDER_GETBLOCK_ACCESS_TOKENS", value = var.getblock_access_tokens },

        { name = "RPC_PROXY_PROVIDER_PROMETHEUS_QUERY_URL", value = "http://127.0.0.1:${local.prometheus_proxy_port}/workspaces/${var.prometheus_workspace_id}" },
        { name = "RPC_PROXY_PROVIDER_PROMETHEUS_WORKSPACE_HEADER", value = "aps-workspaces.${module.this.region}.amazonaws.com" },

        { name = "RPC_PROXY_REGISTRY_API_URL", value = var.registry_api_endpoint },
        { name = "RPC_PROXY_REGISTRY_API_AUTH_TOKEN", value = var.registry_api_auth_token },
        { name = "RPC_PROXY_REGISTRY_PROJECT_DATA_CACHE_TTL", value = tostring(var.project_cache_ttl) },

        { name = "RPC_PROXY_STORAGE_REDIS_MAX_CONNECTIONS", value = tostring(var.redis_max_connections) },
        { name = "RPC_PROXY_STORAGE_PROJECT_DATA_REDIS_ADDR_READ", value = "redis://${var.project_cache_endpoint_read}/0" },
        { name = "RPC_PROXY_STORAGE_PROJECT_DATA_REDIS_ADDR_WRITE", value = "redis://${var.project_cache_endpoint_write}/0" },
        { name = "RPC_PROXY_STORAGE_IDENTITY_CACHE_REDIS_ADDR_READ", value = "redis://${var.identity_cache_endpoint_read}/1" },
        { name = "RPC_PROXY_STORAGE_IDENTITY_CACHE_REDIS_ADDR_WRITE", value = "redis://${var.identity_cache_endpoint_write}/1" },
        { name = "RPC_PROXY_STORAGE_RATE_LIMITING_CACHE_REDIS_ADDR_READ", value = "redis://${var.rate_limiting_cache_endpoint_read}/2" },
        { name = "RPC_PROXY_STORAGE_RATE_LIMITING_CACHE_REDIS_ADDR_WRITE", value = "redis://${var.rate_limiting_cache_endpoint_write}/2" },

        { name = "RPC_PROXY_RATE_LIMITING_MAX_TOKENS", value = tostring(var.rate_limiting_max_tokens) },
        { name = "RPC_PROXY_RATE_LIMITING_REFILL_INTERVAL_SEC", value = tostring(var.rate_limiting_refill_interval) },
        { name = "RPC_PROXY_RATE_LIMITING_REFILL_RATE", value = tostring(var.rate_limiting_refill_rate) },

        { name = "RPC_PROXY_POSTGRES_URI", value = var.postgres_url },

        { name = "RPC_PROXY_ANALYTICS_EXPORT_BUCKET", value = var.analytics_datalake_bucket_name },
      ],

      portMappings = [
        {
          containerPort = var.port,
          hostPort      = var.port
        }
      ],

      ulimits : [
        {
          name      = "nofile",
          softLimit = local.file_descriptor_soft_limit,
          hardLimit = local.file_descriptor_hard_limit,
        }
      ],

      logConfiguration : {
        logDriver = "awslogs",
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.cluster.name,
          "awslogs-region"        = module.this.region,
          "awslogs-stream-prefix" = "ecs"
        }
      },

      dependsOn = [
        { containerName : "aws-otel-collector", condition : "START" },
        { containerName : "sigv4-prometheus-proxy", condition : "START" },
      ]
    },

    # Forward telemetry data to AWS CloudWatch
    {
      name      = "aws-otel-collector",
      image     = "public.ecr.aws/aws-observability/aws-otel-collector:v0.31.0",
      cpu       = local.otel_cpu,
      memory    = local.otel_memory,
      essential = true,

      command = [
        "--config=/etc/ecs/ecs-amp-prometheus.yaml",
        # Uncomment to enable debug logging in otel-collector
        # "--set=service.telemetry.logs.level=DEBUG"
      ],

      environment = [
        { name : "AWS_PROMETHEUS_SCRAPING_ENDPOINT", value : "0.0.0.0:${local.otel_port}" },
        { name : "AWS_PROMETHEUS_ENDPOINT", value : "${var.prometheus_endpoint}api/v1/remote_write" },
        { name : "AWS_REGION", value : module.this.region },
      ],

      logConfiguration = {
        logDriver = "awslogs",
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.otel.name,
          "awslogs-region"        = module.this.region,
          "awslogs-stream-prefix" = "ecs"
        }
      }
    },

    # SigV4 Proxy to sign HTTP requests to Prometheus (for providers weight updates)
    {
      name      = "sigv4-prometheus-proxy",
      image     = "public.ecr.aws/aws-observability/aws-sigv4-proxy:latest",
      cpu       = local.prometheus_proxy_cpu,
      memory    = local.prometheus_proxy_memory
      essential = true,

      portMappings = [
        {
          containerPort = local.prometheus_proxy_port,
          hostPort      = local.prometheus_proxy_port,
        }
      ],

      command = [
        "--port=0.0.0.0:${local.prometheus_proxy_port}"
      ],

      logConfiguration = {
        logDriver = "awslogs",
        options : {
          "awslogs-group"         = aws_cloudwatch_log_group.prometheus_proxy.name,
          "awslogs-region"        = module.this.region,
          "awslogs-stream-prefix" = "ecs"
        }
      },
    }
  ])
}


#-------------------------------------------------------------------------------
# ECS Service

resource "aws_ecs_service" "app_service" {
  name            = "${module.this.id}-service"
  cluster         = aws_ecs_cluster.app_cluster.id
  task_definition = aws_ecs_task_definition.app_task.arn
  launch_type     = "FARGATE"
  desired_count   = local.desired_count
  propagate_tags  = "TASK_DEFINITION"

  # Wait for the service deployment to succeed
  wait_for_steady_state = true

  network_configuration {
    subnets          = var.private_subnets
    assign_public_ip = false
    security_groups  = [aws_security_group.app_ingress.id]
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
