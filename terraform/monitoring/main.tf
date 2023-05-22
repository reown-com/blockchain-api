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
  opsgenie_notification_channel = "NNOynGwVz"
  notifications = (
    var.environment == "prod" ?
    [{ uid = local.opsgenie_notification_channel }] :
    []
  )

  target_group  = split(":", var.target_group_arn)[5]
  load_balancer = join("/", slice(split("/", var.load_balancer_arn), 1, 4))
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
       "annotations": {
          "list": [
            {
              "builtIn": 1,
              "datasource": {
                "type": "datasource",
                "uid": "grafana"
              },
              "enable": true,
              "hide": true,
              "iconColor": "rgba(0, 211, 255, 1)",
              "name": "Annotations & Alerts",
              "target": {
                "limit": 100,
                "matchAny": false,
                "tags": [],
                "type": "dashboard"
              },
              "type": "dashboard"
            }
          ]
        },
        "editable": true,
        "fiscalYearStartMonth": 0,
        "graphTooltip": 0,
        "id": 4,
        "links": [],
        "liveNow": false,
        "panels": [
          {
            "collapsed": false,
            "datasource": {
              "type": "datasource",
              "uid": "grafana"
            },
            "gridPos": {
              "h": 1,
              "w": 24,
              "x": 0,
              "y": 0
            },
            "id": 15,
            "panels": [],
            "targets": [
              {
                "datasource": {
                  "type": "datasource",
                  "uid": "grafana"
                },
                "refId": "A"
              }
            ],
            "title": "ECS",
            "type": "row"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "max": 100,
                "min": 0,
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                },
                "unit": "percent"
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 8,
              "x": 0,
              "y": 1
            },
            "id": 6,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "exemplar": false,
                "expr": "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\",code=~\"5.+\"}[5m])) or vector(0)",
                "hide": true,
                "interval": "",
                "legendFormat": "",
                "refId": "A"
              },
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "exemplar": true,
                "expr": "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
                "hide": true,
                "interval": "",
                "legendFormat": "",
                "refId": "B"
              },
              {
                "datasource": {
                  "type": "__expr__",
                  "uid": "__expr__"
                },
                "expression": "(1-(($A+$C)/$B))*100",
                "hide": false,
                "refId": "Availability",
                "type": "math"
              },
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "exemplar": true,
                "expr": "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\",code=\"429\"}[5m])) or vector(0)",
                "hide": true,
                "interval": "",
                "legendFormat": "",
                "refId": "C"
              }
            ],
            "thresholds": [],
            "title": "Availability",
            "type": "timeseries"
          },
          {
            "alert": {
              "alertRuleTags": {},
              "conditions": [
                {
                  "evaluator": {
                    "params": [
                      70
                    ],
                    "type": "gt"
                  },
                  "operator": {
                    "type": "and"
                  },
                  "query": {
                    "params": [
                      "A",
                      "5m",
                      "now"
                    ]
                  },
                  "reducer": {
                    "params": [],
                    "type": "avg"
                  },
                  "type": "query"
                }
              ],
              "executionErrorState": "alerting",
              "for": "5m",
              "frequency": "1m",
              "handler": 1,
              "message": "RPC Proxy's memory utilization is high (over 70%)",
              "name": "ECS Memory Utilization alert",
              "noDataState": "no_data",
              "notifications": local.notifications
            },
            "datasource": {
              "type": "cloudwatch",
              "uid": grafana_data_source.cloudwatch.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "area"
                  }
                },
                "mappings": [],
                "max": 100,
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "#EAB839",
                      "value": 40
                    },
                    {
                      "color": "red",
                      "value": 70
                    }
                  ]
                },
                "unit": "percent"
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 8,
              "x": 8,
              "y": 1
            },
            "id": 18,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "alias": "",
                "datasource": {
                  "type": "cloudwatch",
                  "uid": grafana_data_source.cloudwatch.uid
                },
                "dimensions": {
                  "ServiceName": "${var.environment}_rpc-proxy-service"
                },
                "expression": "",
                "id": "",
                "matchExact": false,
                "metricEditorMode": 0,
                "metricName": "MemoryUtilization",
                "metricQueryType": 0,
                "namespace": "AWS/ECS",
                "period": "",
                "queryMode": "Metrics",
                "refId": "A",
                "region": "default",
                "sqlExpression": "",
                "statistic": "Average"
              },
              {
                "alias": "",
                "datasource": {
                  "type": "cloudwatch",
                  "uid": grafana_data_source.cloudwatch.uid
                },
                "dimensions": {
                  "ServiceName": "${var.environment}_rpc-proxy-service"
                },
                "expression": "",
                "id": "",
                "matchExact": false,
                "metricEditorMode": 0,
                "metricName": "MemoryUtilization",
                "metricQueryType": 0,
                "namespace": "AWS/ECS",
                "period": "",
                "queryMode": "Metrics",
                "refId": "B",
                "region": "default",
                "sqlExpression": "",
                "statistic": "Maximum"
              }
            ],
            "thresholds": [
              {
                "colorMode": "critical",
                "op": "gt",
                "visible": true
              }
            ],
            "title": "ECS Memory Utilization",
            "type": "timeseries"
          },
          {
            "alert": {
              "alertRuleTags": {},
              "conditions": [
                {
                  "evaluator": {
                    "params": [
                      70
                    ],
                    "type": "gt"
                  },
                  "operator": {
                    "type": "and"
                  },
                  "query": {
                    "params": [
                      "A",
                      "5m",
                      "now"
                    ]
                  },
                  "reducer": {
                    "params": [],
                    "type": "avg"
                  },
                  "type": "query"
                }
              ],
              "executionErrorState": "alerting",
              "for": "5m",
              "frequency": "1m",
              "handler": 1,
              "message": "RPC Proxy's CPU utilization is high (over 70%)",
              "name": "ECS CPU Utilization alert",
              "noDataState": "no_data",
              "notifications": local.notifications
            },
            "datasource": {
              "type": "cloudwatch",
              "uid": grafana_data_source.cloudwatch.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "area"
                  }
                },
                "mappings": [],
                "max": 100,
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "#EAB839",
                      "value": 40
                    },
                    {
                      "color": "red",
                      "value": 70
                    }
                  ]
                },
                "unit": "percent"
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 8,
              "x": 16,
              "y": 1
            },
            "id": 17,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "alias": "",
                "datasource": {
                  "type": "cloudwatch",
                  "uid": grafana_data_source.cloudwatch.uid
                },
                "dimensions": {
                  "ServiceName": "${var.environment}_rpc-proxy-service"
                },
                "expression": "",
                "id": "",
                "matchExact": false,
                "metricEditorMode": 0,
                "metricName": "CPUUtilization",
                "metricQueryType": 0,
                "namespace": "AWS/ECS",
                "period": "",
                "queryMode": "Metrics",
                "refId": "A",
                "region": "default",
                "sqlExpression": "",
                "statistic": "Average"
              },
              {
                "alias": "",
                "datasource": {
                  "type": "cloudwatch",
                  "uid": grafana_data_source.cloudwatch.uid
                },
                "dimensions": {
                  "ServiceName": "${var.environment}_rpc-proxy-service"
                },
                "expression": "",
                "id": "",
                "matchExact": false,
                "metricEditorMode": 0,
                "metricName": "CPUUtilization",
                "metricQueryType": 0,
                "namespace": "AWS/ECS",
                "period": "",
                "queryMode": "Metrics",
                "refId": "B",
                "region": "default",
                "sqlExpression": "",
                "statistic": "Maximum"
              }
            ],
            "thresholds": [
              {
                "colorMode": "critical",
                "op": "gt",
                "value": 70,
                "visible": true
              }
            ],
            "title": "ECS CPU Utilization",
            "type": "timeseries"
          },
          {
            "collapsed": false,
            "gridPos": {
              "h": 1,
              "w": 24,
              "x": 0,
              "y": 10
            },
            "id": 28,
            "panels": [],
            "title": "Status codes",
            "type": "row"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 8,
              "w": 8,
              "x": 0,
              "y": 11
            },
            "id": 21,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "builder",
                "expr": "sum by(status_code) (increase(provider_status_code_counter{provider=\"Infura\"}[$__rate_interval]))",
                "legendFormat": "__auto",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Infura Status Codes",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 8,
              "w": 8,
              "x": 8,
              "y": 11
            },
            "id": 25,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "builder",
                "expr": "sum by(status_code) (increase(provider_status_code_counter{provider=\"zkSync\"}[$__rate_interval]))",
                "legendFormat": "__auto",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "ZkSync Status Codes",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 8,
              "w": 8,
              "x": 16,
              "y": 11
            },
            "id": 23,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "builder",
                "expr": "sum by(status_code) (increase(provider_status_code_counter{provider=\"Publicnode\"}[$__rate_interval]))",
                "legendFormat": "__auto",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Publicnode Status Codes",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 8,
              "w": 8,
              "x": 0,
              "y": 19
            },
            "id": 22,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "builder",
                "expr": "sum by(status_code) (increase(provider_status_code_counter{provider=\"Omniatech\"}[$__rate_interval]))",
                "legendFormat": "__auto",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Omniatech Status Codes",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 8,
              "w": 8,
              "x": 8,
              "y": 19
            },
            "id": 26,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "builder",
                "expr": "sum by(status_code) (increase(provider_status_code_counter{provider=\"Binance\"}[$__rate_interval]))",
                "legendFormat": "__auto",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Binance Status Codes",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 8,
              "w": 8,
              "x": 16,
              "y": 19
            },
            "id": 24,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "builder",
                "expr": "sum by(status_code) (increase(provider_status_code_counter{provider=\"Pokt\"}[$__rate_interval]))",
                "legendFormat": "__auto",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Pokt Status Codes",
            "type": "timeseries"
          },
          {
            "collapsed": false,
            "datasource": {
              "type": "datasource",
              "uid": "grafana"
            },
            "gridPos": {
              "h": 1,
              "w": 24,
              "x": 0,
              "y": 27
            },
            "id": 13,
            "panels": [],
            "targets": [
              {
                "datasource": {
                  "type": "datasource",
                  "uid": "grafana"
                },
                "refId": "A"
              }
            ],
            "title": "Proxy metrics",
            "type": "row"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 12,
              "x": 0,
              "y": 28
            },
            "id": 2,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "builder",
                "exemplar": false,
                "expr": "sum by(chain_id) (increase(rpc_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
                "interval": "",
                "legendFormat": "",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Calls by Chain ID",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                },
                "unit": "µs"
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 12,
              "x": 12,
              "y": 28
            },
            "id": 5,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "code",
                "exemplar": false,
                "expr": "histogram_quantile(0.95, sum(rate(http_external_latency_tracker_bucket{}[$__rate_interval])) by (le, provider))",
                "interval": "",
                "legendFormat": "",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Latency values",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 8,
              "x": 0,
              "y": 37
            },
            "id": 9,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "code",
                "exemplar": true,
                "expr": "round(sum(increase(http_call_counter{code=~\"5.+\"}[5m])))",
                "hide": true,
                "interval": "",
                "legendFormat": "",
                "range": true,
                "refId": "A"
              },
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "code",
                "expr": "round(sum(increase(http_call_counter{code=\"502\"}[5m])))",
                "hide": true,
                "legendFormat": "__auto",
                "range": true,
                "refId": "B"
              },
              {
                "datasource": {
                  "name": "Expression",
                  "type": "__expr__",
                  "uid": "__expr__"
                },
                "expression": "$A-$B",
                "hide": false,
                "refId": "5xx",
                "type": "math"
              }
            ],
            "thresholds": [],
            "title": "Non provider errors",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 8,
              "x": 8,
              "y": 37
            },
            "id": 19,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "code",
                "expr": "round(sum(increase(http_call_counter{code=\"502\"}[5m])))",
                "hide": false,
                "legendFormat": "__auto",
                "range": true,
                "refId": "B"
              }
            ],
            "thresholds": [],
            "title": "Provider Errors",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 8,
              "x": 16,
              "y": 37
            },
            "id": 7,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "editorMode": "code",
                "expr": "sum (increase(rejected_project_counter[5m]))",
                "legendFormat": "__auto",
                "range": true,
                "refId": "A"
              }
            ],
            "title": "Rejected project ID",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 10,
              "x": 0,
              "y": 46
            },
            "id": 4,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "exemplar": false,
                "expr": "sum by (code)(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
                "interval": "",
                "legendFormat": "",
                "refId": "A"
              }
            ],
            "title": "HTTP Response Codes",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "cloudwatch",
              "uid": grafana_data_source.cloudwatch.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 3,
              "x": 10,
              "y": 46
            },
            "id": 15,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "alias": "eu-central-1",
                "datasource": {
                  "type": "cloudwatch",
                  "uid": grafana_data_source.cloudwatch.uid
                },
                "dimensions": {
                  "TargetGroup": "targetgroup/${var.environment}-rpc-proxy-055/d55fc6acac3c3c72"
                },
                "expression": "",
                "id": "",
                "matchExact": false,
                "metricEditorMode": 0,
                "metricName": "HealthyHostCount",
                "metricQueryType": 1,
                "namespace": "AWS/NetworkELB",
                "period": "",
                "queryMode": "Metrics",
                "refId": "A",
                "region": "default",
                "sql": {
                  "from": {
                    "property": {
                      "name": "AWS/NetworkELB",
                      "type": "string"
                    },
                    "type": "property"
                  },
                  "select": {
                    "name": "MAX",
                    "parameters": [
                      {
                        "name": "HealthyHostCount",
                        "type": "functionParameter"
                      }
                    ],
                    "type": "function"
                  },
                  "where": {
                    "expressions": [
                      {
                        "operator": {
                          "name": "=",
                          "value": "net/${var.environment}-rpc-proxy-lb-c77/8bc3437271f7cf0b"
                        },
                        "property": {
                          "name": "LoadBalancer",
                          "type": "string"
                        },
                        "type": "operator"
                      }
                    ],
                    "type": "and"
                  }
                },
                "sqlExpression": "SELECT MAX(HealthyHostCount) FROM \"AWS/NetworkELB\" WHERE LoadBalancer = 'net/${var.environment}-rpc-proxy-lb-c77/8bc3437271f7cf0b'",
                "statistic": "Maximum"
              }
            ],
            "title": "Healthy Hosts",
            "type": "timeseries"
          },
          {
            "datasource": {
              "type": "prometheus",
              "uid": grafana_data_source.prometheus.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "off"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 80
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 11,
              "x": 13,
              "y": 46
            },
            "id": 3,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "datasource": {
                  "type": "prometheus",
                  "uid": grafana_data_source.prometheus.uid
                },
                "exemplar": false,
                "expr": "sum by (chain_id)(rate(rpc_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
                "interval": "",
                "legendFormat": "",
                "refId": "A"
              }
            ],
            "title": "Calls by Chain ID",
            "type": "timeseries"
          },
          {
            "collapsed": false,
            "datasource": {
              "type": "datasource",
              "uid": "grafana"
            },
            "gridPos": {
              "h": 1,
              "w": 24,
              "x": 0,
              "y": 55
            },
            "id": 11,
            "panels": [],
            "targets": [
              {
                "datasource": {
                  "type": "datasource",
                  "uid": "grafana"
                },
                "refId": "A"
              }
            ],
            "title": "Database",
            "type": "row"
          },
          {
            "alert": {
              "alertRuleTags": {
                "priority": "P2"
              },
              "conditions": [
                {
                  "evaluator": {
                    "params": [
                      50
                    ],
                    "type": "gt"
                  },
                  "operator": {
                    "type": "or"
                  },
                  "query": {
                    "params": [
                      "B",
                      "5m",
                      "now"
                    ]
                  },
                  "reducer": {
                    "params": [],
                    "type": "max"
                  },
                  "type": "query"
                },
                {
                  "evaluator": {
                    "params": [
                      50
                    ],
                    "type": "gt"
                  },
                  "operator": {
                    "type": "or"
                  },
                  "query": {
                    "params": [
                      "A",
                      "5m",
                      "now"
                    ]
                  },
                  "reducer": {
                    "params": [],
                    "type": "max"
                  },
                  "type": "query"
                }
              ],
              "executionErrorState": "alerting",
              "for": "5m",
              "frequency": "1m",
              "handler": 1,
              "message": "${var.environment} CPU/Memory alert",
              "name": "${var.environment} CPU/Memory alert",
              "noDataState": "alerting",
              "notifications": local.notifications
            },
            "datasource": {
              "type": "cloudwatch",
              "uid": grafana_data_source.cloudwatch.uid
            },
            "fieldConfig": {
              "defaults": {
                "color": {
                  "mode": "palette-classic"
                },
                "custom": {
                  "axisCenteredZero": false,
                  "axisColorMode": "text",
                  "axisLabel": "",
                  "axisPlacement": "auto",
                  "axisSoftMax": 100,
                  "axisSoftMin": 0,
                  "barAlignment": 0,
                  "drawStyle": "line",
                  "fillOpacity": 0,
                  "gradientMode": "none",
                  "hideFrom": {
                    "legend": false,
                    "tooltip": false,
                    "viz": false
                  },
                  "lineInterpolation": "linear",
                  "lineWidth": 1,
                  "pointSize": 5,
                  "scaleDistribution": {
                    "type": "linear"
                  },
                  "showPoints": "auto",
                  "spanNulls": false,
                  "stacking": {
                    "group": "A",
                    "mode": "none"
                  },
                  "thresholdsStyle": {
                    "mode": "area"
                  }
                },
                "mappings": [],
                "thresholds": {
                  "mode": "absolute",
                  "steps": [
                    {
                      "color": "green",
                      "value": null
                    },
                    {
                      "color": "red",
                      "value": 50
                    }
                  ]
                }
              },
              "overrides": []
            },
            "gridPos": {
              "h": 9,
              "w": 12,
              "x": 0,
              "y": 56
            },
            "id": 8,
            "options": {
              "legend": {
                "calcs": [],
                "displayMode": "list",
                "placement": "bottom",
                "showLegend": true
              },
              "tooltip": {
                "mode": "single",
                "sort": "none"
              }
            },
            "targets": [
              {
                "alias": "",
                "datasource": {
                  "type": "cloudwatch",
                  "uid": grafana_data_source.cloudwatch.uid
                },
                "dimensions": {
                  "CacheClusterId": "rpc-proxy-rpc-${var.environment}"
                },
                "expression": "",
                "id": "",
                "matchExact": true,
                "metricEditorMode": 0,
                "metricName": "CPUUtilization",
                "metricQueryType": 0,
                "namespace": "AWS/ElastiCache",
                "period": "",
                "queryMode": "Metrics",
                "refId": "A",
                "region": "default",
                "sqlExpression": "",
                "statistic": "Maximum"
              },
              {
                "alias": "",
                "datasource": {
                  "type": "cloudwatch",
                  "uid": grafana_data_source.cloudwatch.uid
                },
                "dimensions": {
                  "CacheClusterId": "rpc-proxy-rpc-${var.environment}"
                },
                "expression": "",
                "hide": false,
                "id": "",
                "matchExact": true,
                "metricEditorMode": 0,
                "metricName": "DatabaseMemoryUsagePercentage",
                "metricQueryType": 0,
                "namespace": "AWS/ElastiCache",
                "period": "",
                "queryMode": "Metrics",
                "refId": "B",
                "region": "default",
                "sqlExpression": "",
                "statistic": "Maximum"
              }
            ],
            "title": "Redis CPU/Memory",
            "type": "timeseries"
          }
        ],
        "refresh": "",
        "revision": 1,
        "schemaVersion": 38,
        "style": "dark",
        "tags": [],
        "templating": {
          "list": []
        },
        "time": {
          "from": "now-6h",
          "to": "now"
        },
        "timepicker": {},
        "timezone": "",
        "title": "${var.environment}_rpc-proxy",
        "uid": "${var.environment}_rpc-proxy",
        "version": 13,
        "weekStart": ""
      }
        )
}
