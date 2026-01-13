# Hetzner Runner Module - Outputs
# Export useful information about the created runner

output "server_id" {
  description = "The unique ID of the Hetzner server"
  value       = hcloud_server.runner.id
}

output "server_name" {
  description = "The name of the server"
  value       = hcloud_server.runner.name
}

output "server_status" {
  description = "The current status of the server"
  value       = hcloud_server.runner.status
}

output "ipv4_address" {
  description = "The public IPv4 address of the server"
  value       = hcloud_server.runner.ipv4_address
}

output "ipv6_address" {
  description = "The public IPv6 address of the server"
  value       = var.enable_ipv6 ? hcloud_server.runner.ipv6_address : null
}

output "ipv6_network" {
  description = "The IPv6 network of the server"
  value       = var.enable_ipv6 ? hcloud_server.runner.ipv6_network : null
}

output "datacenter" {
  description = "The datacenter where the server is located"
  value       = hcloud_server.runner.datacenter
}

output "server_type" {
  description = "The server type"
  value       = hcloud_server.runner.server_type
}

output "labels" {
  description = "The labels attached to the server"
  value       = hcloud_server.runner.labels
}

output "ssh_command" {
  description = "SSH command to connect to the server"
  value       = "ssh root@${hcloud_server.runner.ipv4_address}"
}

output "firewall_id" {
  description = "The ID of the firewall (if created by module)"
  value       = length(var.firewall_ids) == 0 ? hcloud_firewall.runner[0].id : null
}

output "runner_config" {
  description = "Runner configuration summary"
  value = {
    name        = var.name
    repo        = var.github_repo
    labels      = var.runner_labels
    group       = var.runner_group
    environment = var.environment
  }
}
