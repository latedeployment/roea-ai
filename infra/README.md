# roea-ai Infrastructure

Infrastructure as Code (IaC) for roea-ai cloud resources using Terraform.

## Overview

This directory contains Terraform configurations for:
- **Hetzner Cloud**: Linux CI/CD runners
- **GitHub Actions**: Self-hosted runner setup
- **DNS/CDN**: Website infrastructure (future)

## Prerequisites

- [Terraform](https://www.terraform.io/downloads) >= 1.5.0
- [Hetzner Cloud API Token](https://console.hetzner.cloud/)
- [GitHub Personal Access Token](https://github.com/settings/tokens) with `repo` and `admin:org` scopes

## Directory Structure

```
infra/
├── README.md                 # This file
├── modules/
│   └── hetzner-runner/       # Reusable Hetzner runner module
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
├── environments/
│   ├── dev/                  # Development environment
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── terraform.tfvars.example
│   └── prod/                 # Production environment
│       ├── main.tf
│       ├── variables.tf
│       └── terraform.tfvars.example
├── scripts/
│   ├── setup-runner.sh       # GitHub Actions runner setup
│   └── bootstrap.sh          # Initial state backend setup
└── .terraform.lock.hcl       # Provider lock file
```

## Quick Start

### 1. Set Up Backend (First Time)

```bash
# Create S3-compatible bucket for state (Hetzner Object Storage or AWS S3)
./scripts/bootstrap.sh
```

### 2. Configure Environment

```bash
cd environments/dev

# Copy example variables
cp terraform.tfvars.example terraform.tfvars

# Edit with your values
vim terraform.tfvars
```

### 3. Deploy

```bash
# Initialize Terraform
terraform init

# Preview changes
terraform plan

# Apply changes
terraform apply
```

## Environment Variables

Required environment variables:

| Variable | Description |
|----------|-------------|
| `HCLOUD_TOKEN` | Hetzner Cloud API token |
| `GITHUB_TOKEN` | GitHub PAT for runner registration |

Optional:

| Variable | Description |
|----------|-------------|
| `TF_VAR_github_repo` | Override target repository |
| `TF_VAR_runner_count` | Number of runners to create |

## Modules

### hetzner-runner

Creates a Hetzner Cloud VM configured as a GitHub Actions self-hosted runner.

```hcl
module "runner" {
  source = "../modules/hetzner-runner"

  name           = "roea-runner-1"
  server_type    = "cx21"  # 2 vCPU, 4GB RAM
  image          = "ubuntu-22.04"
  ssh_keys       = ["my-ssh-key"]
  github_repo    = "your-org/roea-ai"
  runner_labels  = ["linux", "x64", "self-hosted"]
}
```

**Inputs:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `name` | string | - | Server name |
| `server_type` | string | `cx21` | Hetzner server type |
| `image` | string | `ubuntu-22.04` | OS image |
| `location` | string | `nbg1` | Datacenter location |
| `ssh_keys` | list(string) | `[]` | SSH key names |
| `github_repo` | string | - | Repository for runner |
| `runner_labels` | list(string) | `[]` | Runner labels |

**Outputs:**

| Name | Description |
|------|-------------|
| `server_id` | Hetzner server ID |
| `server_ip` | Public IPv4 address |
| `server_status` | Server status |

## CI/CD Integration

Infrastructure changes are validated on every PR:

```yaml
# .github/workflows/infra.yml
name: Infrastructure
on:
  pull_request:
    paths: ['infra/**']
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: hashicorp/setup-terraform@v3
      - run: terraform fmt -check
      - run: terraform validate
```

See `.github/workflows/infra.yml` for the full workflow.

## Cost Estimates

| Resource | Type | Monthly Cost |
|----------|------|--------------|
| Linux Runner | CX21 (2 vCPU, 4GB) | ~€5.18 |
| Linux Runner | CX31 (2 vCPU, 8GB) | ~€8.98 |
| E2E Runner | CCX13 (2 dedicated, 8GB) | ~€14.40 |

Hetzner pricing is very competitive for CI/CD workloads.

## Security Considerations

- API tokens are passed via environment variables, never committed
- SSH keys are managed via Hetzner console
- GitHub runner tokens are short-lived (auto-generated)
- State file contains sensitive data - use encrypted backend

## Troubleshooting

### Runner Not Connecting

1. Check runner service status: `systemctl status actions.runner.*`
2. Verify GitHub token has correct permissions
3. Check firewall rules (outbound HTTPS required)

### Terraform State Lock

If state is locked:
```bash
terraform force-unlock <lock-id>
```

### Provider Issues

Update providers:
```bash
terraform init -upgrade
```

## Contributing

When making infrastructure changes:

1. Create a feature branch
2. Update Terraform code
3. Run `terraform fmt` and `terraform validate`
4. Open PR - CI will validate
5. Get approval before applying

Never apply directly to production without PR review.
