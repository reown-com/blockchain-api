locals {
  opsgenie_notification_channel = "NNOynGwVz"
  notifications = (
    var.environment == "prod" ?
    [{ uid = local.opsgenie_notification_channel }] :
    []
  )

  target_group = split(":", var.target_group_arn)[5]

  # Turns the arn into the format expected by the Grafana provider e.g.
  # net/prod-relay-load-balancer/e9a51c46020a0f85
  load_balancer = join("/", slice(split("/", var.load_balancer_arn), 1, 4))
}
