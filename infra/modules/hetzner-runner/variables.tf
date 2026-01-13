# Hetzner Runner Module - Variables
# Terraform variables for GitHub Actions self-hosted runner on Hetzner Cloud

variable "name" {
  description = "Name of the server (must be unique)"
  type        = string
}

variable "server_type" {
  description = "Hetzner server type (e.g., cx21, cx31, ccx13)"
  type        = string
  default     = "cx21"

  validation {
    condition = can(regex("^(cx|cpx|ccx|cax)[0-9]+$", var.server_type))
    error_message = "Server type must be a valid Hetzner type (cx21, cpx11, ccx13, cax11, etc.)."
  }
}

variable "image" {
  description = "OS image to use"
  type        = string
  default     = "ubuntu-22.04"
}

variable "location" {
  description = "Hetzner datacenter location"
  type        = string
  default     = "nbg1"

  validation {
    condition     = contains(["nbg1", "fsn1", "hel1", "ash", "hil"], var.location)
    error_message = "Location must be one of: nbg1, fsn1, hel1, ash, hil."
  }
}

variable "ssh_keys" {
  description = "List of SSH key names or IDs for server access"
  type        = list(string)
  default     = []
}

variable "github_repo" {
  description = "GitHub repository for runner registration (format: owner/repo)"
  type        = string

  validation {
    condition     = can(regex("^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$", var.github_repo))
    error_message = "GitHub repo must be in format 'owner/repo'."
  }
}

variable "runner_labels" {
  description = "Labels to apply to the GitHub Actions runner"
  type        = list(string)
  default     = ["self-hosted", "linux", "x64"]
}

variable "runner_group" {
  description = "Runner group to add the runner to (optional)"
  type        = string
  default     = "default"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"

  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be one of: dev, staging, prod."
  }
}

variable "enable_ipv6" {
  description = "Enable IPv6 on the server"
  type        = bool
  default     = true
}

variable "backups" {
  description = "Enable automatic backups (additional cost)"
  type        = bool
  default     = false
}

variable "firewall_ids" {
  description = "List of firewall IDs to attach to the server"
  type        = list(number)
  default     = []
}

variable "user_data_extra" {
  description = "Additional cloud-init user data to append"
  type        = string
  default     = ""
}

variable "runner_version" {
  description = "GitHub Actions runner version (leave empty for latest)"
  type        = string
  default     = ""
}

variable "docker_enabled" {
  description = "Install Docker on the runner"
  type        = bool
  default     = true
}

variable "rust_toolchain" {
  description = "Rust toolchain to install (empty to skip)"
  type        = string
  default     = "stable"
}

variable "node_version" {
  description = "Node.js version to install (empty to skip)"
  type        = string
  default     = "20"
}
