# Production Environment - Main Configuration
# Terraform configuration for production GitHub Actions runners
# Includes standard runners and dedicated E2E test runners

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.45"
    }
  }

  # Backend configuration for remote state
  # REQUIRED for production - configure after running bootstrap.sh
  # backend "s3" {
  #   bucket                      = "roea-ai-terraform-state"
  #   key                         = "prod/terraform.tfstate"
  #   region                      = "eu-central-1"
  #   endpoint                    = "https://fsn1.your-objectstorage.com"
  #   skip_credentials_validation = true
  #   skip_metadata_api_check     = true
  #   skip_region_validation      = true
  #   force_path_style            = true
  #   encrypt                     = true
  # }
}

# Configure the Hetzner Cloud provider
provider "hcloud" {
  token = var.hcloud_token
}

# Local values for consistent naming
locals {
  environment = "prod"
  name_prefix = "roea-${local.environment}"

  common_labels = ["self-hosted", "linux", "x64", local.environment]
  e2e_labels    = ["self-hosted", "linux", "x64", local.environment, "e2e", "dedicated"]
}

# Standard Linux runners
module "linux_runner" {
  source   = "../../modules/hetzner-runner"
  count    = var.runner_count

  name           = "${local.name_prefix}-runner-${count.index + 1}"
  server_type    = var.runner_type
  image          = "ubuntu-22.04"
  # Distribute runners across locations for redundancy
  location       = count.index % 2 == 0 ? var.location : var.secondary_location
  ssh_keys       = var.ssh_key_names
  github_repo    = var.github_repo
  runner_labels  = concat(local.common_labels, ["runner-${count.index + 1}"])
  environment    = local.environment

  # Production settings
  docker_enabled = true
  rust_toolchain = "stable"
  node_version   = "20"
  backups        = true  # Enable backups for prod
}

# Dedicated E2E test runners with more resources
module "e2e_runner" {
  source   = "../../modules/hetzner-runner"
  count    = var.e2e_runner_count

  name           = "${local.name_prefix}-e2e-${count.index + 1}"
  server_type    = var.e2e_runner_type
  image          = "ubuntu-22.04"
  location       = var.location  # Keep E2E runners in primary location
  ssh_keys       = var.ssh_key_names
  github_repo    = var.github_repo
  runner_labels  = concat(local.e2e_labels, ["e2e-${count.index + 1}"])
  environment    = local.environment

  # E2E runners need more tooling
  docker_enabled = true
  rust_toolchain = "stable"
  node_version   = "20"
  backups        = true

  # Additional packages for E2E testing
  user_data_extra = <<-EOF
    # Install additional E2E dependencies
    - apt-get install -y xvfb libgtk-3-0 libwebkit2gtk-4.1-0 libayatana-appindicator3-1
    # Install Playwright dependencies
    - sudo -u runner npx playwright install-deps || true
  EOF
}

# Output all runner information
output "standard_runners" {
  description = "Information about standard runners"
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

output "e2e_runners" {
  description = "Information about E2E test runners"
  value = {
    for idx, runner in module.e2e_runner : runner.server_name => {
      id         = runner.server_id
      ip         = runner.ipv4_address
      status     = runner.server_status
      ssh        = runner.ssh_command
      datacenter = runner.datacenter
    }
  }
}

output "all_runner_ips" {
  description = "List of all runner IP addresses"
  value = concat(
    [for runner in module.linux_runner : runner.ipv4_address],
    [for runner in module.e2e_runner : runner.ipv4_address]
  )
}

output "environment" {
  description = "Environment name"
  value       = local.environment
}

output "runner_summary" {
  description = "Summary of runners created"
  value = {
    standard_count = var.runner_count
    e2e_count      = var.e2e_runner_count
    total          = var.runner_count + var.e2e_runner_count
    locations      = distinct(concat(
      [for runner in module.linux_runner : runner.datacenter],
      [for runner in module.e2e_runner : runner.datacenter]
    ))
  }
}
