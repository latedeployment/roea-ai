# Production Environment - Variables
# Input variables for the production environment

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
  description = "Number of standard Linux runners"
  type        = number
  default     = 2

  validation {
    condition     = var.runner_count >= 1 && var.runner_count <= 10
    error_message = "Runner count must be between 1 and 10 for prod environment."
  }
}

variable "e2e_runner_count" {
  description = "Number of dedicated E2E test runners"
  type        = number
  default     = 1

  validation {
    condition     = var.e2e_runner_count >= 0 && var.e2e_runner_count <= 3
    error_message = "E2E runner count must be between 0 and 3."
  }
}

variable "runner_type" {
  description = "Hetzner server type for standard runners"
  type        = string
  default     = "cx31"
}

variable "e2e_runner_type" {
  description = "Hetzner server type for E2E runners (dedicated CPU recommended)"
  type        = string
  default     = "ccx13"
}

variable "location" {
  description = "Primary Hetzner datacenter location"
  type        = string
  default     = "fsn1"
}

variable "secondary_location" {
  description = "Secondary datacenter location for redundancy"
  type        = string
  default     = "nbg1"
}
