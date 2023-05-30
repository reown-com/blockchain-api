output "dashboard_definition" {
  description = "The JSON definition of the dashboard."
  value       = data.jsonnet_file.dashboard.rendered
}
