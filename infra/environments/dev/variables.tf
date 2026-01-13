# Development Environment - Variables
# Input variables for the dev environment

variable "hcloud_token" {
  description = "Hetzner Cloud API token"
  type        = string
  sensitive   = true
}

variable "github_token" {
  description = "GitHub Personal Access Token for runner registration"
  type        = string
  sensitive   = true
  default     = ""
}

variable "github_repo" {
  description = "GitHub repository for runner (format: owner/repo)"
  type        = string
  default     = "your-org/roea-ai"
}

variable "ssh_key_names" {
  description = "Names of SSH keys registered in Hetzner Cloud"
  type        = list(string)
  default     = []
}

variable "runner_count" {
  description = "Number of Linux runners to create"
  type        = number
  default     = 1

  validation {
    condition     = var.runner_count >= 0 && var.runner_count <= 5
    error_message = "Runner count must be between 0 and 5 for dev environment."
  }
}

variable "runner_type" {
  description = "Hetzner server type for runners"
  type        = string
  default     = "cx21"
}

variable "location" {
  description = "Hetzner datacenter location"
  type        = string
  default     = "nbg1"
}
