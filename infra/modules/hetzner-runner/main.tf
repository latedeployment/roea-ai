# Hetzner Runner Module - Main Configuration
# Creates a Hetzner Cloud server configured as a GitHub Actions self-hosted runner

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.45"
    }
  }
}

# Generate cloud-init user data for runner setup
locals {
  runner_labels_str = join(",", var.runner_labels)

  user_data = <<-EOF
    #cloud-config
    package_update: true
    package_upgrade: true

    packages:
      - curl
      - wget
      - git
      - jq
      - build-essential
      - pkg-config
      - libssl-dev
      - unzip
      - htop

    write_files:
      - path: /opt/runner-config.env
        permissions: '0600'
        content: |
          GITHUB_REPO=${var.github_repo}
          RUNNER_LABELS=${local.runner_labels_str}
          RUNNER_GROUP=${var.runner_group}
          RUNNER_NAME=${var.name}
          ENVIRONMENT=${var.environment}

      - path: /opt/setup-runner.sh
        permissions: '0755'
        content: |
          #!/bin/bash
          set -euo pipefail

          # Load configuration
          source /opt/runner-config.env

          # Create runner user
          useradd -m -s /bin/bash runner || true
          mkdir -p /home/runner/actions-runner
          chown -R runner:runner /home/runner

          # Get registration token from GitHub
          # Note: GITHUB_TOKEN must be set as environment variable
          if [ -z "$${GITHUB_TOKEN:-}" ]; then
            echo "GITHUB_TOKEN not set, skipping runner registration"
            exit 0
          fi

          TOKEN=$(curl -s -X POST \
            -H "Authorization: token $${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github+json" \
            "https://api.github.com/repos/$${GITHUB_REPO}/actions/runners/registration-token" \
            | jq -r '.token')

          if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
            echo "Failed to get registration token"
            exit 1
          fi

          # Download latest runner
          cd /home/runner/actions-runner
          RUNNER_VERSION="${var.runner_version}"
          if [ -z "$RUNNER_VERSION" ]; then
            RUNNER_VERSION=$(curl -s https://api.github.com/repos/actions/runner/releases/latest | jq -r '.tag_name' | sed 's/v//')
          fi

          curl -o actions-runner-linux-x64.tar.gz -L \
            "https://github.com/actions/runner/releases/download/v$${RUNNER_VERSION}/actions-runner-linux-x64-$${RUNNER_VERSION}.tar.gz"
          tar xzf actions-runner-linux-x64.tar.gz
          rm actions-runner-linux-x64.tar.gz
          chown -R runner:runner /home/runner/actions-runner

          # Configure runner
          sudo -u runner ./config.sh \
            --url "https://github.com/$${GITHUB_REPO}" \
            --token "$${TOKEN}" \
            --name "$${RUNNER_NAME}" \
            --labels "$${RUNNER_LABELS}" \
            --runnergroup "$${RUNNER_GROUP}" \
            --unattended \
            --replace

          # Install and start service
          ./svc.sh install runner
          ./svc.sh start

          echo "Runner setup complete!"

    runcmd:
      %{if var.docker_enabled}
      # Install Docker
      - curl -fsSL https://get.docker.com | sh
      - usermod -aG docker runner || true
      - systemctl enable docker
      - systemctl start docker
      %{endif}

      %{if var.rust_toolchain != ""}
      # Install Rust
      - |
        sudo -u runner bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${var.rust_toolchain}'
        echo 'source $HOME/.cargo/env' >> /home/runner/.bashrc
      %{endif}

      %{if var.node_version != ""}
      # Install Node.js via nvm
      - |
        sudo -u runner bash -c 'curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash'
        sudo -u runner bash -c 'export NVM_DIR="$HOME/.nvm" && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" && nvm install ${var.node_version}'
      %{endif}

      # Run custom user data
      ${indent(6, var.user_data_extra)}

      # Setup runner (if token available via user-data)
      - /opt/setup-runner.sh || echo "Runner setup skipped - run manually with GITHUB_TOKEN"

    final_message: "Cloud-init complete. Runner: ${var.name}"
  EOF
}

# Create the server
resource "hcloud_server" "runner" {
  name        = var.name
  server_type = var.server_type
  image       = var.image
  location    = var.location

  ssh_keys    = var.ssh_keys
  user_data   = local.user_data
  backups     = var.backups

  firewall_ids = var.firewall_ids

  public_net {
    ipv4_enabled = true
    ipv6_enabled = var.enable_ipv6
  }

  labels = {
    environment = var.environment
    role        = "github-runner"
    managed_by  = "terraform"
    repo        = replace(var.github_repo, "/", "-")
  }

  lifecycle {
    # Prevent accidental destruction
    prevent_destroy = false

    # Ignore changes to user_data after creation
    ignore_changes = [user_data]
  }
}

# Optional: Create a dedicated firewall for the runner
resource "hcloud_firewall" "runner" {
  count = length(var.firewall_ids) == 0 ? 1 : 0
  name  = "${var.name}-firewall"

  # Allow SSH
  rule {
    direction  = "in"
    protocol   = "tcp"
    port       = "22"
    source_ips = ["0.0.0.0/0", "::/0"]
  }

  # Allow ICMP (ping)
  rule {
    direction  = "in"
    protocol   = "icmp"
    source_ips = ["0.0.0.0/0", "::/0"]
  }

  # All outbound traffic allowed by default in Hetzner

  labels = {
    environment = var.environment
    managed_by  = "terraform"
  }
}

# Attach firewall if we created one
resource "hcloud_firewall_attachment" "runner" {
  count       = length(var.firewall_ids) == 0 ? 1 : 0
  firewall_id = hcloud_firewall.runner[0].id
  server_ids  = [hcloud_server.runner.id]
}
