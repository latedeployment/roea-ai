# Development Environment - Main Configuration
# Terraform configuration for dev environment GitHub Actions runners

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.45"
    }
  }

  # Backend configuration for remote state
  # Uncomment and configure after running bootstrap.sh
  # backend "s3" {
  #   bucket                      = "roea-ai-terraform-state"
  #   key                         = "dev/terraform.tfstate"
  #   region                      = "eu-central-1"
  #   endpoint                    = "https://fsn1.your-objectstorage.com"
  #   skip_credentials_validation = true
  #   skip_metadata_api_check     = true
  #   skip_region_validation      = true
  #   force_path_style            = true
  # }
}

# Configure the Hetzner Cloud provider
provider "hcloud" {
  token = var.hcloud_token
}

# Local values for consistent naming
locals {
  environment = "dev"
  name_prefix = "roea-${local.environment}"

  common_labels = ["self-hosted", "linux", "x64", local.environment]
}

# Create Linux runners using the module
module "linux_runner" {
  source   = "../../modules/hetzner-runner"
  count    = var.runner_count

  name           = "${local.name_prefix}-runner-${count.index + 1}"
  server_type    = var.runner_type
  image          = "ubuntu-22.04"
  location       = var.location
  ssh_keys       = var.ssh_key_names
  github_repo    = var.github_repo
  runner_labels  = concat(local.common_labels, ["runner-${count.index + 1}"])
  environment    = local.environment

  # Dev environment settings
  docker_enabled = true
  rust_toolchain = "stable"
  node_version   = "20"
  backups        = false
}

# Output all runner information
output "runners" {
  description = "Information about created runners"
  value = {
    for idx, runner in module.linux_runner : runner.server_name => {
      id         = runner.server_id
      ip         = runner.ipv4_address
      status     = runner.server_status
      ssh        = runner.ssh_command
      datacenter = runner.datacenter
    }
  }
}

output "runner_ips" {
  description = "List of runner IP addresses"
  value       = [for runner in module.linux_runner : runner.ipv4_address]
}

output "environment" {
  description = "Environment name"
  value       = local.environment
}
