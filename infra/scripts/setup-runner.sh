#!/bin/bash
# Manual runner setup script
# Use this to configure a GitHub Actions runner on an existing server

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Default values
RUNNER_DIR="/home/runner/actions-runner"
RUNNER_USER="runner"

# Display usage
usage() {
    cat << EOF
Usage: $0 --repo <owner/repo> --token <github-token> [OPTIONS]

Configure a GitHub Actions self-hosted runner.

Required:
    --repo <owner/repo>     GitHub repository (e.g., your-org/roea-ai)
    --token <token>         GitHub Personal Access Token with repo scope

Options:
    --name <name>           Runner name (default: hostname)
    --labels <labels>       Comma-separated labels (default: self-hosted,linux,x64)
    --group <group>         Runner group (default: default)
    --work-dir <dir>        Work directory (default: /home/runner/actions-runner)
    --user <user>           User to run as (default: runner)
    --replace               Replace existing runner with same name
    --uninstall             Uninstall runner instead of installing
    --help                  Show this help message

Examples:
    # Basic setup
    $0 --repo your-org/roea-ai --token ghp_xxx

    # Custom configuration
    $0 --repo your-org/roea-ai --token ghp_xxx \\
       --name my-runner --labels linux,x64,rust,e2e

    # Uninstall runner
    $0 --repo your-org/roea-ai --token ghp_xxx --uninstall

EOF
}

# Parse arguments
GITHUB_REPO=""
GITHUB_TOKEN=""
RUNNER_NAME="$(hostname)"
RUNNER_LABELS="self-hosted,linux,x64"
RUNNER_GROUP="default"
REPLACE_RUNNER=false
UNINSTALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --repo)
            GITHUB_REPO="$2"
            shift 2
            ;;
        --token)
            GITHUB_TOKEN="$2"
            shift 2
            ;;
        --name)
            RUNNER_NAME="$2"
            shift 2
            ;;
        --labels)
            RUNNER_LABELS="$2"
            shift 2
            ;;
        --group)
            RUNNER_GROUP="$2"
            shift 2
            ;;
        --work-dir)
            RUNNER_DIR="$2"
            shift 2
            ;;
        --user)
            RUNNER_USER="$2"
            shift 2
            ;;
        --replace)
            REPLACE_RUNNER=true
            shift
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate required arguments
if [ -z "$GITHUB_REPO" ]; then
    log_error "--repo is required"
    usage
    exit 1
fi

if [ -z "$GITHUB_TOKEN" ]; then
    log_error "--token is required"
    usage
    exit 1
fi

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root"
        exit 1
    fi
}

# Create runner user if needed
create_user() {
    log_step "Creating runner user: $RUNNER_USER"

    if id "$RUNNER_USER" &>/dev/null; then
        log_info "User $RUNNER_USER already exists"
    else
        useradd -m -s /bin/bash "$RUNNER_USER"
        log_info "Created user: $RUNNER_USER"
    fi
}

# Get registration token from GitHub
get_registration_token() {
    log_step "Getting registration token from GitHub..."

    local response
    response=$(curl -s -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/$GITHUB_REPO/actions/runners/registration-token")

    REG_TOKEN=$(echo "$response" | jq -r '.token')

    if [ "$REG_TOKEN" = "null" ] || [ -z "$REG_TOKEN" ]; then
        log_error "Failed to get registration token"
        echo "Response: $response"
        exit 1
    fi

    log_info "Registration token obtained"
}

# Get removal token from GitHub
get_removal_token() {
    log_step "Getting removal token from GitHub..."

    local response
    response=$(curl -s -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/$GITHUB_REPO/actions/runners/remove-token")

    REMOVE_TOKEN=$(echo "$response" | jq -r '.token')

    if [ "$REMOVE_TOKEN" = "null" ] || [ -z "$REMOVE_TOKEN" ]; then
        log_error "Failed to get removal token"
        echo "Response: $response"
        exit 1
    fi

    log_info "Removal token obtained"
}

# Download and extract runner
download_runner() {
    log_step "Downloading GitHub Actions runner..."

    mkdir -p "$RUNNER_DIR"
    cd "$RUNNER_DIR"

    # Get latest version
    RUNNER_VERSION=$(curl -s https://api.github.com/repos/actions/runner/releases/latest | jq -r '.tag_name' | sed 's/v//')
    log_info "Latest runner version: $RUNNER_VERSION"

    # Download
    local runner_url="https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"
    log_info "Downloading from: $runner_url"

    curl -o actions-runner-linux-x64.tar.gz -L "$runner_url"
    tar xzf actions-runner-linux-x64.tar.gz
    rm actions-runner-linux-x64.tar.gz

    chown -R "$RUNNER_USER:$RUNNER_USER" "$RUNNER_DIR"
    log_info "Runner downloaded and extracted"
}

# Configure runner
configure_runner() {
    log_step "Configuring runner..."

    cd "$RUNNER_DIR"

    local config_args=(
        --url "https://github.com/$GITHUB_REPO"
        --token "$REG_TOKEN"
        --name "$RUNNER_NAME"
        --labels "$RUNNER_LABELS"
        --runnergroup "$RUNNER_GROUP"
        --unattended
    )

    if [ "$REPLACE_RUNNER" = true ]; then
        config_args+=(--replace)
    fi

    sudo -u "$RUNNER_USER" ./config.sh "${config_args[@]}"
    log_info "Runner configured"
}

# Install service
install_service() {
    log_step "Installing runner service..."

    cd "$RUNNER_DIR"
    ./svc.sh install "$RUNNER_USER"
    ./svc.sh start

    log_info "Runner service installed and started"
}

# Uninstall runner
uninstall_runner() {
    log_step "Uninstalling runner..."

    cd "$RUNNER_DIR"

    # Stop service
    if [ -f "./svc.sh" ]; then
        ./svc.sh stop || true
        ./svc.sh uninstall || true
    fi

    # Remove runner
    get_removal_token
    sudo -u "$RUNNER_USER" ./config.sh remove --token "$REMOVE_TOKEN" || true

    log_info "Runner uninstalled"
}

# Print summary
print_summary() {
    echo ""
    echo "======================================"
    echo "  Runner Setup Complete!"
    echo "======================================"
    echo ""
    echo "  Name:     $RUNNER_NAME"
    echo "  Repo:     $GITHUB_REPO"
    echo "  Labels:   $RUNNER_LABELS"
    echo "  Group:    $RUNNER_GROUP"
    echo "  User:     $RUNNER_USER"
    echo "  Dir:      $RUNNER_DIR"
    echo ""
    echo "Commands:"
    echo "  Status:   systemctl status actions.runner.*"
    echo "  Logs:     journalctl -u actions.runner.* -f"
    echo "  Stop:     cd $RUNNER_DIR && ./svc.sh stop"
    echo "  Start:    cd $RUNNER_DIR && ./svc.sh start"
    echo ""
}

# Main execution
main() {
    echo "======================================"
    echo "  GitHub Actions Runner Setup"
    echo "======================================"
    echo ""

    check_root

    if [ "$UNINSTALL" = true ]; then
        uninstall_runner
        log_info "Uninstall complete!"
        exit 0
    fi

    create_user
    get_registration_token
    download_runner
    configure_runner
    install_service
    print_summary
}

main
