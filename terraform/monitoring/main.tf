terraform {
  required_version = "~> 1.0"

  required_providers {
    grafana = {
      source  = "grafana/grafana"
      version = "~> 1.24"
    }
  }
}

resource "grafana_data_source" "prometheus" {
  type = "prometheus"
  name = "${var.environment}-rpc-proxy-amp"
  url  = "https://aps-workspaces.eu-central-1.amazonaws.com/workspaces/${var.prometheus_workspace_id}/"

  json_data {
    http_method     = "GET"
    sigv4_auth      = true
    sigv4_auth_type = "workspace-iam-role"
    sigv4_region    = "eu-central-1"
  }
}

resource "grafana_data_source" "cloudwatch" {
  type = "cloudwatch"
  name = "${var.environment}-rpc-proxy-cloudwatch"

  json_data {
    default_region = "eu-central-1"
  }
}

# JSON Dashboard. When exporting from Grafana make sure that all
# variables are replaced properly
resource "grafana_dashboard" "at_a_glance" {
  overwrite = true
  message   = "Updated by Terraform"
  config_json = jsonencode({
    annotations : {
      list : [
        {
          builtIn : 1,
          datasource : "-- Grafana --",
          enable : true,
          hide : true,
          iconColor : "rgba(0, 211, 255, 1)",
          name : "Annotations & Alerts",
          target : {
            limit : 100,
            matchAny : false,
            tags : [],
            type : "dashboard"
          },
          type : "dashboard"
        }
      ]
    },
    editable : true,
    fiscalYearStartMonth : 0,
    graphTooltip : 0,
    id : 19,
    links : [],
    liveNow : false,
    panels : [
      {
        datasource : {
          type : "prometheus",
          uid : grafana_data_source.prometheus.uid
        },
        fieldConfig : {
          defaults : {
            color : {
              mode : "palette-classic"
            },
            custom : {
              axisLabel : "",
              axisPlacement : "auto",
              barAlignment : 0,
              drawStyle : "line",
              fillOpacity : 0,
              gradientMode : "none",
              hideFrom : {
                legend : false,
                tooltip : false,
                viz : false
              },
              lineInterpolation : "linear",
              lineWidth : 1,
              pointSize : 5,
              scaleDistribution : {
                type : "linear"
              },
              showPoints : "auto",
              spanNulls : false,
              stacking : {
                group : "A",
                mode : "none"
              },
              thresholdsStyle : {
                mode : "off"
              }
            },
            mappings : [],
            thresholds : {
              mode : "absolute",
              steps : [
                {
                  color : "green",
                  value : null
                },
                {
                  color : "red",
                  value : 80
                }
              ]
            }
          },
          overrides : []
        },
        gridPos : {
          h : 9,
          w : 12,
          x : 0,
          y : 0
        },
        id : 2,
        options : {
          legend : {
            calcs : [],
            displayMode : "list",
            placement : "bottom"
          },
          tooltip : {
            mode : "single",
            sort : "none"
          }
        },
        targets : [
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : false,
            expr : "sum by (chain_id) (rpc_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"})",
            interval : "",
            legendFormat : "",
            refId : "A"
          }
        ],
        title : "Calls by Chain ID",
        type : "timeseries"
      },
      {
        datasource : {
          type : "prometheus",
          uid : grafana_data_source.prometheus.uid
        },
        fieldConfig : {
          defaults : {
            color : {
              mode : "palette-classic"
            },
            custom : {
              axisLabel : "",
              axisPlacement : "auto",
              barAlignment : 0,
              drawStyle : "line",
              fillOpacity : 0,
              gradientMode : "none",
              hideFrom : {
                legend : false,
                tooltip : false,
                viz : false
              },
              lineInterpolation : "linear",
              lineWidth : 1,
              pointSize : 5,
              scaleDistribution : {
                type : "linear"
              },
              showPoints : "auto",
              spanNulls : false,
              stacking : {
                group : "A",
                mode : "none"
              },
              thresholdsStyle : {
                mode : "off"
              }
            },
            mappings : [],
            thresholds : {
              mode : "absolute",
              steps : [
                {
                  color : "green",
                  value : null
                },
                {
                  color : "red",
                  value : 80
                }
              ]
            }
          },
          overrides : []
        },
        gridPos : {
          h : 9,
          w : 12,
          x : 12,
          y : 0
        },
        id : 4,
        options : {
          legend : {
            calcs : [],
            displayMode : "list",
            placement : "bottom"
          },
          tooltip : {
            mode : "single",
            sort : "none"
          }
        },
        targets : [
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : false,
            expr : "sum by (chain_id)(rate(rpc_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
            interval : "",
            legendFormat : "",
            refId : "A"
          }
        ],
        title : "Calls by Chain ID",
        type : "timeseries"
      },
      {
        datasource : {
          type : "prometheus",
          uid : grafana_data_source.prometheus.uid
        },
        fieldConfig : {
          defaults : {
            color : {
              mode : "palette-classic"
            },
            custom : {
              axisLabel : "",
              axisPlacement : "auto",
              barAlignment : 0,
              drawStyle : "line",
              fillOpacity : 0,
              gradientMode : "none",
              hideFrom : {
                legend : false,
                tooltip : false,
                viz : false
              },
              lineInterpolation : "linear",
              lineWidth : 1,
              pointSize : 5,
              scaleDistribution : {
                type : "linear"
              },
              showPoints : "auto",
              spanNulls : false,
              stacking : {
                group : "A",
                mode : "none"
              },
              thresholdsStyle : {
                mode : "off"
              }
            },
            mappings : [],
            thresholds : {
              mode : "absolute",
              steps : [
                {
                  color : "green",
                  value : null
                },
                {
                  color : "red",
                  value : 80
                }
              ]
            }
          },
          overrides : []
        },
        gridPos : {
          h : 9,
          w : 12,
          x : 0,
          y : 9
        },
        id : 5,
        options : {
          legend : {
            calcs : [],
            displayMode : "list",
            placement : "bottom"
          },
          tooltip : {
            mode : "single",
            sort : "none"
          }
        },
        targets : [
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : false,
            expr : "sum by (code)(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
            interval : "",
            legendFormat : "",
            refId : "A"
          }
        ],
        title : "Calls by Chain ID",
        type : "timeseries"
      },
      {
        datasource : {
          type : "prometheus",
          uid : grafana_data_source.prometheus.uid
        },
        fieldConfig : {
          defaults : {
            color : {
              mode : "palette-classic"
            },
            custom : {
              axisLabel : "",
              axisPlacement : "auto",
              barAlignment : 0,
              drawStyle : "line",
              fillOpacity : 0,
              gradientMode : "none",
              hideFrom : {
                legend : false,
                tooltip : false,
                viz : false
              },
              lineInterpolation : "linear",
              lineWidth : 1,
              pointSize : 5,
              scaleDistribution : {
                type : "linear"
              },
              showPoints : "auto",
              spanNulls : false,
              stacking : {
                group : "A",
                mode : "none"
              },
              thresholdsStyle : {
                mode : "off"
              }
            },
            mappings : [],
            thresholds : {
              mode : "absolute",
              steps : [
                {
                  color : "green",
                  value : null
                },
                {
                  color : "red",
                  value : 80
                }
              ]
            },
            unit : "dtdurations"
          },
          overrides : []
        },
        gridPos : {
          h : 9,
          w : 12,
          x : 12,
          y : 9
        },
        id : 6,
        options : {
          legend : {
            calcs : [],
            displayMode : "list",
            placement : "bottom"
          },
          tooltip : {
            mode : "single",
            sort : "none"
          }
        },
        targets : [
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : false,
            expr : "sum by (route)(rate(http_latency_tracker{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
            interval : "",
            legendFormat : "",
            refId : "A"
          }
        ],
        title : "Latency",
        type : "timeseries"
      },
      {
        datasource : {
          type : "prometheus",
          uid : grafana_data_source.prometheus.uid
        },
        fieldConfig : {
          defaults : {
            color : {
              mode : "palette-classic"
            },
            custom : {
              axisLabel : "",
              axisPlacement : "auto",
              barAlignment : 0,
              drawStyle : "line",
              fillOpacity : 0,
              gradientMode : "none",
              hideFrom : {
                legend : false,
                tooltip : false,
                viz : false
              },
              lineInterpolation : "linear",
              lineWidth : 1,
              pointSize : 5,
              scaleDistribution : {
                type : "linear"
              },
              showPoints : "auto",
              spanNulls : false,
              stacking : {
                group : "A",
                mode : "none"
              },
              thresholdsStyle : {
                mode : "off"
              }
            },
            mappings : [],
            max : 100,
            min : 0,
            thresholds : {
              mode : "absolute",
              steps : [
                {
                  color : "green",
                  value : null
                },
                {
                  color : "red",
                  value : 80
                }
              ]
            },
            unit : "percent"
          },
          overrides : []
        },
        gridPos : {
          h : 9,
          w : 12,
          x : 12,
          y : 9
        },
        id : 6,
        options : {
          legend : {
            calcs : [],
            displayMode : "list",
            placement : "bottom"
          },
          tooltip : {
            mode : "single",
            sort : "none"
          }
        },
        targets : [
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : false,
            expr : "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\",code=~\"5.+\"}[5m])) or vector(0)",
            hide : true,
            interval : "",
            legendFormat : "",
            refId : "A"
          },
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : true,
            expr : "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
            hide : true,
            interval : "",
            legendFormat : "",
            refId : "B"
          },
          {
            datasource : {
              type : "__expr__",
              uid : "__expr__"
            },
            expression : "(1-(($A+$C)/$B))*100",
            hide : false,
            refId : "Availability",
            type : "math"
          },
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : true,
            expr : "sum(rate(http_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\",code=\"429\"}[5m])) or vector(0)",
            hide : true,
            interval : "",
            legendFormat : "",
            refId : "C"
          }
        ],
        thresholds : [],
        title : "Availability",
        type : "timeseries"
      },

      {
        datasource : {
          type : "prometheus",
          uid : grafana_data_source.prometheus.uid
        },
        fieldConfig : {
          defaults : {
            color : {
              mode : "palette-classic"
            },
            custom : {
              axisLabel : "",
              axisPlacement : "auto",
              barAlignment : 0,
              drawStyle : "line",
              fillOpacity : 0,
              gradientMode : "none",
              hideFrom : {
                legend : false,
                tooltip : false,
                viz : false
              },
              lineInterpolation : "linear",
              lineWidth : 1,
              pointSize : 5,
              scaleDistribution : {
                type : "linear"
              },
              showPoints : "auto",
              spanNulls : false,
              stacking : {
                group : "A",
                mode : "none"
              },
              thresholdsStyle : {
                mode : "off"
              }
            },
            mappings : [],
            thresholds : {
              mode : "absolute",
              steps : [
                {
                  color : "green",
                  value : null
                },
                {
                  color : "red",
                  value : 80
                }
              ]
            }
          },
          overrides : []
        },
        gridPos : {
          h : 8,
          w : 9,
          x : 0,
          y : 0
        },
        id : 50,
        options : {
          legend : {
            calcs : [],
            displayMode : "list",
            placement : "bottom"
          },
          tooltip : {
            mode : "single",
            sort : "none"
          }
        },
        targets : [
          {
            datasource : {
              type : "prometheus",
              uid : grafana_data_source.prometheus.uid
            },
            exemplar : true,
            expr : "sum by (chain_id) (rpc_call_counter{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"})",
            expr : "sum by (status_code)(rate(http_requests_total{aws_ecs_task_family=\"${var.environment}_rpc-proxy\"}[5m]))",
            interval : "",
            legendFormat : "",
            refId : "A"
          }
        ],
        title : "Registry Requests by Status Code",
        type : "timeseries"
      }
    ],
    schemaVersion : 36,
    style : "dark",
    tags : [],
    templating : {
      list : []
    },
    time : {
      from : "now-6h",
      to : "now"
    },
    timepicker : {},
    timezone : "",
    title : "${var.environment}_rpc-proxy",
    uid : "${var.environment}_rpc-proxy",
    version : 1,
    weekStart : ""
  })
}
