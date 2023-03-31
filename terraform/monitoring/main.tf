terraform {
  required_version = "~> 1.0"

  required_providers {
    grafana = {
      source  = "grafana/grafana"
      version = "~> 1.31"
    }
  }
}

locals {
  opsgenie_notification_channel = "l_iaPw6nk"
  notifications = (
    var.environment == "prod" ?
    "[{\"uid\": \"${local.opsgenie_notification_channel}\"}]" :
    "[]"
  )
}

resource "grafana_data_source" "prometheus" {
  type = "prometheus"
  name = "${var.environment}-rpc-proxy-amp"
  url  = "https://aps-workspaces.eu-central-1.amazonaws.com/workspaces/${var.prometheus_workspace_id}/"

  json_data_encoded = jsonencode({
    httpMethod    = "GET"
    manageAlerts  = false
    sigV4Auth     = true
    sigV4AuthType = "ec2_iam_role"
    sigV4Region   = "eu-central-1"
  })
}

resource "grafana_data_source" "cloudwatch" {
  type = "cloudwatch"
  name = "${var.environment}-rpc-proxy-cloudwatch"

  json_data_encoded = jsonencode({
    defaultRegion = "eu-central-1"
  })
}

# JSON Dashboard. When exporting from Grafana make sure that all
# variables are replaced properly
resource "grafana_dashboard" "at_a_glance" {
  overwrite = true
  message   = "Updated by Terraform"
  config_json = jsonencode(
    {
      "annotations" : {
        "list" : [
          {
            "builtIn" : 1,
            "datasource" : "-- Grafana --",
            "enable" : true,
            "hide" : true,
            "iconColor" : "rgba(0, 211, 255, 1)",
            "name" : "Annotations & Alerts",
            "target" : {
              "limit" : 100,
              "matchAny" : false,
              "tags" : [],
              "type" : "dashboard"
            },
            "type" : "dashboard"
          }
        ]
      },
      "editable" : true,
      "fiscalYearStartMonth" : 0,
      "graphTooltip" : 0,
      "id" : 22,
      "links" : [],
      "liveNow" : false,
      "panels" : [
        {
          "gridPos" : {
            "h" : 1,
            "w" : 24,
            "x" : 0,
            "y" : 0
          },
          "id" : 15,
          "title" : "ECS",
          "type" : "row"
        },
        {
          "datasource" : {
            "type" : "prometheus",
            "uid" : grafana_data_source.prometheus.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "off"
                }
              },
              "mappings" : [],
              "max" : 100,
              "min" : 0,
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 80
                  }
                ]
              },
              "unit" : "percent"
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 0,
            "y" : 1
          },
          "id" : 6,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : false,
              "expr" : "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\",code=~\"5.+\"}[5m])) or vector(0)",
              "hide" : true,
              "interval" : "",
              "legendFormat" : "",
              "refId" : "A"
            },
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : true,
              "expr" : "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
              "hide" : true,
              "interval" : "",
              "legendFormat" : "",
              "refId" : "B"
            },
            {
              "datasource" : {
                "type" : "__expr__",
                "uid" : "__expr__"
              },
              "expression" : "(1-(($A+$C)/$B))*100",
              "hide" : false,
              "refId" : "Availability",
              "type" : "math"
            },
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : true,
              "expr" : "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\",code=\"429\"}[5m])) or vector(0)",
              "hide" : true,
              "interval" : "",
              "legendFormat" : "",
              "refId" : "C"
            }
          ],
          "thresholds" : [],
          "title" : "Availability",
          "type" : "timeseries"
        },
        {
          "alert" : {
            "alertRuleTags" : {},
            "conditions" : [
              {
                "evaluator" : {
                  "params" : [
                    70
                  ],
                  "type" : "gt"
                },
                "operator" : {
                  "type" : "and"
                },
                "query" : {
                  "params" : [
                    "A",
                    "5m",
                    "now"
                  ]
                },
                "reducer" : {
                  "params" : [],
                  "type" : "avg"
                },
                "type" : "query"
              }
            ],
            "executionErrorState" : "alerting",
            "for" : "5m",
            "frequency" : "1m",
            "handler" : 1,
            "message" : "RPC Proxy's CPU utilization is high (over 70%)",
            "name" : "ECS CPU Utilization alert",
            "noDataState" : "no_data",
            "notifications" : local.notifications
          },
          "datasource" : {
            "type" : "cloudwatch",
            "uid" : grafana_data_source.cloudwatch.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "area"
                }
              },
              "mappings" : [],
              "max" : 100,
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "#EAB839",
                    "value" : 40
                  },
                  {
                    "color" : "red",
                    "value" : 70
                  }
                ]
              },
              "unit" : "percent"
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 12,
            "y" : 1
          },
          "id" : 17,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "alias" : "",
              "datasource" : {
                "type" : "cloudwatch",
                "uid" : grafana_data_source.cloudwatch.uid
              },
              "dimensions" : {
                "ServiceName" : "${var.environment}_rpc-proxy-service"
              },
              "expression" : "",
              "id" : "",
              "matchExact" : false,
              "metricEditorMode" : 0,
              "metricName" : "CPUUtilization",
              "metricQueryType" : 0,
              "namespace" : "AWS/ECS",
              "period" : "",
              "queryMode" : "Metrics",
              "refId" : "A",
              "region" : "default",
              "sqlExpression" : "",
              "statistic" : "Average"
            }
          ],
          "thresholds" : [
            {
              "colorMode" : "critical",
              "op" : "gt",
              "value" : 70,
              "visible" : true
            }
          ],
          "title" : "ECS CPU Utilization",
          "type" : "timeseries"
        },
        {
          "alert" : {
            "alertRuleTags" : {},
            "conditions" : [
              {
                "evaluator" : {
                  "params" : [
                    null
                  ],
                  "type" : "gt"
                },
                "operator" : {
                  "type" : "and"
                },
                "query" : {
                  "params" : [
                    "A",
                    "5m",
                    "now"
                  ]
                },
                "reducer" : {
                  "params" : [],
                  "type" : "avg"
                },
                "type" : "query"
              }
            ],
            "executionErrorState" : "alerting",
            "for" : "5m",
            "frequency" : "1m",
            "handler" : 1,
            "message" : "RPC Proxy's memory utilization is high (over 70%)",
            "name" : "ECS Memory Utilization alert",
            "noDataState" : "no_data",
            "notifications" : local.notifications
          },
          "datasource" : {
            "type" : "cloudwatch",
            "uid" : grafana_data_source.cloudwatch.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "area"
                }
              },
              "mappings" : [],
              "max" : 100,
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "#EAB839",
                    "value" : 40
                  },
                  {
                    "color" : "red",
                    "value" : 70
                  }
                ]
              },
              "unit" : "percent"
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 0,
            "y" : 10
          },
          "id" : 18,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "alias" : "",
              "datasource" : {
                "type" : "cloudwatch",
                "uid" : grafana_data_source.cloudwatch.uid
              },
              "dimensions" : {
                "ServiceName" : "${var.environment}_rpc-proxy-service"
              },
              "expression" : "",
              "id" : "",
              "matchExact" : false,
              "metricEditorMode" : 0,
              "metricName" : "MemoryUtilization",
              "metricQueryType" : 0,
              "namespace" : "AWS/ECS",
              "period" : "",
              "queryMode" : "Metrics",
              "refId" : "A",
              "region" : "default",
              "sqlExpression" : "",
              "statistic" : "Average"
            }
          ],
          "thresholds" : [
            {
              "colorMode" : "critical",
              "op" : "gt",
              "visible" : true
            }
          ],
          "title" : "ECS Memory Utilization",
          "type" : "timeseries"
        },
        {
          "collapsed" : true,
          "gridPos" : {
            "h" : 1,
            "w" : 24,
            "x" : 0,
            "y" : 19
          },
          "id" : 13,
          "panels" : [],
          "title" : "Proxy metrics",
          "type" : "row"
        },
        {
          "datasource" : {
            "type" : "prometheus",
            "uid" : grafana_data_source.prometheus.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "off"
                }
              },
              "mappings" : [],
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 80
                  }
                ]
              }
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 0,
            "y" : 20
          },
          "id" : 2,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : false,
              "expr" : "sum by (chain_id) (rpc_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"})",
              "interval" : "",
              "legendFormat" : "",
              "refId" : "A"
            }
          ],
          "title" : "Calls by Chain ID",
          "type" : "timeseries"
        },
        {
          "datasource" : {
            "type" : "prometheus",
            "uid" : grafana_data_source.prometheus.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "off"
                }
              },
              "mappings" : [],
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 80
                  }
                ]
              },
              "unit" : "dtdurations"
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 12,
            "y" : 20
          },
          "id" : 5,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : false,
              "expr" : "sum by (route)(rate(http_latency_tracker{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
              "interval" : "",
              "legendFormat" : "",
              "refId" : "A"
            }
          ],
          "title" : "Latency",
          "type" : "timeseries"
        },
        {
          "alert" : {
            "alertRuleTags" : {},
            "conditions" : [
              {
                "evaluator" : {
                  "params" : [
                    20
                  ],
                  "type" : "gt"
                },
                "operator" : {
                  "type" : "and"
                },
                "query" : {
                  "params" : [
                    "A",
                    "5m",
                    "now"
                  ]
                },
                "reducer" : {
                  "params" : [],
                  "type" : "max"
                },
                "type" : "query"
              }
            ],
            "executionErrorState" : "alerting",
            "for" : "5m",
            "frequency" : "1m",
            "handler" : 1,
            "name" : "${var.environment} RPC Proxy Errors alert",
            "noDataState" : "no_data",
            "notifications" : local.notifications
          },
          "datasource" : {
            "type" : "prometheus",
            "uid" : grafana_data_source.prometheus.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "off"
                }
              },
              "mappings" : [],
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 80
                  }
                ]
              }
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 0,
            "y" : 29
          },
          "id" : 9,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : true,
              "expr" : "round(sum(increase(http_call_counter{code=~\"5.+\"}[5m])))",
              "hide" : false,
              "interval" : "",
              "legendFormat" : "",
              "refId" : "A"
            }
          ],
          "thresholds" : [
            {
              "colorMode" : "critical",
              "op" : "gt",
              "value" : 20,
              "visible" : true
            }
          ],
          "title" : "Errors",
          "type" : "timeseries"
        },
        {
          "datasource" : {
            "type" : "prometheus",
            "uid" : grafana_data_source.prometheus.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "off"
                }
              },
              "mappings" : [],
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 80
                  }
                ]
              }
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 12,
            "y" : 29
          },
          "id" : 7,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : true,
              "expr" : "sum by (status_code)(rate(http_requests_total{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
              "interval" : "",
              "legendFormat" : "",
              "refId" : "A"
            }
          ],
          "title" : "Registry Requests by Status Code",
          "type" : "timeseries"
        },
        {
          "datasource" : {
            "type" : "prometheus",
            "uid" : grafana_data_source.prometheus.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "off"
                }
              },
              "mappings" : [],
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 80
                  }
                ]
              }
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 0,
            "y" : 38
          },
          "id" : 4,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : false,
              "expr" : "sum by (code)(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
              "interval" : "",
              "legendFormat" : "",
              "refId" : "A"
            }
          ],
          "title" : "HTTP Response Codes",
          "type" : "timeseries"
        },
        {
          "datasource" : {
            "type" : "prometheus",
            "uid" : grafana_data_source.prometheus.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "off"
                }
              },
              "mappings" : [],
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 80
                  }
                ]
              }
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 12,
            "y" : 38
          },
          "id" : 3,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "datasource" : {
                "type" : "prometheus",
                "uid" : grafana_data_source.prometheus.uid
              },
              "exemplar" : false,
              "expr" : "sum by (chain_id)(rate(rpc_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
              "interval" : "",
              "legendFormat" : "",
              "refId" : "A"
            }
          ],
          "title" : "Calls by Chain ID",
          "type" : "timeseries"
        },
        {
          "collapsed" : false,
          "gridPos" : {
            "h" : 1,
            "w" : 24,
            "x" : 0,
            "y" : 47
          },
          "id" : 11,
          "panels" : [],
          "title" : "Database",
          "type" : "row"
        },
        {
          "alert" : {
            "alertRuleTags" : {
              "priority" : "P2"
            },
            "conditions" : [
              {
                "evaluator" : {
                  "params" : [
                    50
                  ],
                  "type" : "gt"
                },
                "operator" : {
                  "type" : "or"
                },
                "query" : {
                  "params" : [
                    "B",
                    "5m",
                    "now"
                  ]
                },
                "reducer" : {
                  "params" : [],
                  "type" : "max"
                },
                "type" : "query"
              },
              {
                "evaluator" : {
                  "params" : [
                    50
                  ],
                  "type" : "gt"
                },
                "operator" : {
                  "type" : "or"
                },
                "query" : {
                  "params" : [
                    "A",
                    "5m",
                    "now"
                  ]
                },
                "reducer" : {
                  "params" : [],
                  "type" : "max"
                },
                "type" : "query"
              }
            ],
            "executionErrorState" : "alerting",
            "for" : "5m",
            "frequency" : "1m",
            "handler" : 1,
            "message" : "${var.environment} CPU/Memory alert",
            "name" : "${var.environment} CPU/Memory alert",
            "noDataState" : "alerting",
            "notifications" : local.notifications
          },
          "datasource" : {
            "type" : "cloudwatch",
            "uid" : grafana_data_source.cloudwatch.uid
          },
          "fieldConfig" : {
            "defaults" : {
              "color" : {
                "mode" : "palette-classic"
              },
              "custom" : {
                "axisLabel" : "",
                "axisPlacement" : "auto",
                "axisSoftMax" : 100,
                "axisSoftMin" : 0,
                "barAlignment" : 0,
                "drawStyle" : "line",
                "fillOpacity" : 0,
                "gradientMode" : "none",
                "hideFrom" : {
                  "legend" : false,
                  "tooltip" : false,
                  "viz" : false
                },
                "lineInterpolation" : "linear",
                "lineWidth" : 1,
                "pointSize" : 5,
                "scaleDistribution" : {
                  "type" : "linear"
                },
                "showPoints" : "auto",
                "spanNulls" : false,
                "stacking" : {
                  "group" : "A",
                  "mode" : "none"
                },
                "thresholdsStyle" : {
                  "mode" : "area"
                }
              },
              "mappings" : [],
              "thresholds" : {
                "mode" : "absolute",
                "steps" : [
                  {
                    "color" : "green",
                    "value" : null
                  },
                  {
                    "color" : "red",
                    "value" : 50
                  }
                ]
              }
            },
            "overrides" : []
          },
          "gridPos" : {
            "h" : 9,
            "w" : 12,
            "x" : 0,
            "y" : 48
          },
          "id" : 8,
          "options" : {
            "legend" : {
              "calcs" : [],
              "displayMode" : "list",
              "placement" : "bottom"
            },
            "tooltip" : {
              "mode" : "single",
              "sort" : "none"
            }
          },
          "targets" : [
            {
              "alias" : "",
              "datasource" : {
                "type" : "cloudwatch",
                "uid" : grafana_data_source.cloudwatch.uid
              },
              "dimensions" : {
                "CacheClusterId" : var.redis_cluster_id
              },
              "expression" : "",
              "id" : "",
              "matchExact" : true,
              "metricEditorMode" : 0,
              "metricName" : "CPUUtilization",
              "metricQueryType" : 0,
              "namespace" : "AWS/ElastiCache",
              "period" : "",
              "queryMode" : "Metrics",
              "refId" : "A",
              "region" : "default",
              "sqlExpression" : "",
              "statistic" : "Maximum"
            },
            {
              "alias" : "",
              "datasource" : {
                "type" : "cloudwatch",
                "uid" : grafana_data_source.cloudwatch.uid
              },
              "dimensions" : {
                "CacheClusterId" : var.redis_cluster_id
              },
              "expression" : "",
              "hide" : false,
              "id" : "",
              "matchExact" : true,
              "metricEditorMode" : 0,
              "metricName" : "DatabaseMemoryUsagePercentage",
              "metricQueryType" : 0,
              "namespace" : "AWS/ElastiCache",
              "period" : "",
              "queryMode" : "Metrics",
              "refId" : "B",
              "region" : "default",
              "sqlExpression" : "",
              "statistic" : "Maximum"
            }
          ],
          "title" : "Redis CPU/Memory",
          "type" : "timeseries"
        }
      ],
      "schemaVersion" : 35,
      "style" : "dark",
      "tags" : [],
      "templating" : {
        "list" : []
      },
      "time" : {
        "from" : "now-6h",
        "to" : "now"
      },
      "timepicker" : {},
      "timezone" : "",
      "title" : "${var.environment}_rpc-proxy",
      "uid" : "${var.environment}_rpc-proxy",
      "version" : 25,
      "weekStart" : ""
    }
  )
}
