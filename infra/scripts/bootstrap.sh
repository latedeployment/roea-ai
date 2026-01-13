#!/bin/bash
# Bootstrap script for Terraform state backend
# Creates S3-compatible storage for Terraform state files

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check requirements
check_requirements() {
    log_info "Checking requirements..."

    if ! command -v terraform &> /dev/null; then
        log_error "Terraform is not installed. Please install Terraform >= 1.5.0"
        exit 1
    fi

    TERRAFORM_VERSION=$(terraform version -json | jq -r '.terraform_version')
    log_info "Found Terraform version: $TERRAFORM_VERSION"

    if [ -z "${HCLOUD_TOKEN:-}" ]; then
        log_error "HCLOUD_TOKEN environment variable is not set"
        echo "Set it with: export HCLOUD_TOKEN='your-api-token'"
        exit 1
    fi

    log_info "All requirements met!"
}

# Display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Bootstrap Terraform state backend for roea-ai infrastructure.

Options:
    --provider <provider>   State backend provider (hetzner, aws, local)
                           Default: local (for initial setup)
    --bucket <name>        Bucket name for remote state
                           Default: roea-ai-terraform-state
    --region <region>      Region for state storage
                           Default: eu-central-1
    --help                 Show this help message

Examples:
    # Start with local state (recommended for initial setup)
    $0 --provider local

    # Use Hetzner Object Storage
    $0 --provider hetzner --bucket roea-ai-terraform-state

    # Use AWS S3
    $0 --provider aws --bucket roea-ai-terraform-state --region us-east-1

EOF
}

# Parse arguments
PROVIDER="local"
BUCKET_NAME="roea-ai-terraform-state"
REGION="eu-central-1"

while [[ $# -gt 0 ]]; do
    case $1 in
        --provider)
            PROVIDER="$2"
            shift 2
            ;;
        --bucket)
            BUCKET_NAME="$2"
            shift 2
            ;;
        --region)
            REGION="$2"
            shift 2
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

# Setup local backend
setup_local() {
    log_info "Setting up local state backend..."

    # Create local state directories
    mkdir -p "$INFRA_DIR/state/dev"
    mkdir -p "$INFRA_DIR/state/prod"

    # Add to gitignore
    if ! grep -q "state/" "$INFRA_DIR/.gitignore" 2>/dev/null; then
        echo "state/" >> "$INFRA_DIR/.gitignore"
        log_info "Added state/ to .gitignore"
    fi

    log_info "Local state backend ready!"
    echo ""
    log_warn "Local state is suitable for development only."
    log_warn "For production, use remote state with 'hetzner' or 'aws' provider."
}

# Setup Hetzner Object Storage
setup_hetzner() {
    log_info "Setting up Hetzner Object Storage backend..."
    log_warn "Hetzner Object Storage must be created manually via console."
    echo ""
    echo "Steps to create Object Storage:"
    echo "1. Go to https://console.hetzner.cloud/"
    echo "2. Select your project"
    echo "3. Go to 'Storage' -> 'Object Storage'"
    echo "4. Create a new bucket named: $BUCKET_NAME"
    echo "5. Generate access credentials"
    echo ""
    echo "Then set these environment variables:"
    echo "  export AWS_ACCESS_KEY_ID='your-access-key'"
    echo "  export AWS_SECRET_ACCESS_KEY='your-secret-key'"
    echo ""
    echo "Uncomment the backend configuration in main.tf files."
}

# Setup AWS S3
setup_aws() {
    log_info "Setting up AWS S3 backend..."

    if ! command -v aws &> /dev/null; then
        log_error "AWS CLI is not installed"
        exit 1
    fi

    # Create S3 bucket
    log_info "Creating S3 bucket: $BUCKET_NAME"
    aws s3 mb "s3://$BUCKET_NAME" --region "$REGION" || true

    # Enable versioning
    log_info "Enabling versioning..."
    aws s3api put-bucket-versioning \
        --bucket "$BUCKET_NAME" \
        --versioning-configuration Status=Enabled

    # Enable encryption
    log_info "Enabling server-side encryption..."
    aws s3api put-bucket-encryption \
        --bucket "$BUCKET_NAME" \
        --server-side-encryption-configuration '{
            "Rules": [{
                "ApplyServerSideEncryptionByDefault": {
                    "SSEAlgorithm": "AES256"
                }
            }]
        }'

    # Block public access
    log_info "Blocking public access..."
    aws s3api put-public-access-block \
        --bucket "$BUCKET_NAME" \
        --public-access-block-configuration '{
            "BlockPublicAcls": true,
            "IgnorePublicAcls": true,
            "BlockPublicPolicy": true,
            "RestrictPublicBuckets": true
        }'

    log_info "AWS S3 backend ready!"
    echo ""
    echo "Update the backend configuration in main.tf files:"
    echo "  bucket = \"$BUCKET_NAME\""
    echo "  region = \"$REGION\""
}

# Main execution
main() {
    echo "======================================"
    echo "  roea-ai Infrastructure Bootstrap"
    echo "======================================"
    echo ""

    check_requirements

    case $PROVIDER in
        local)
            setup_local
            ;;
        hetzner)
            setup_hetzner
            ;;
        aws)
            setup_aws
            ;;
        *)
            log_error "Unknown provider: $PROVIDER"
            usage
            exit 1
            ;;
    esac

    echo ""
    log_info "Bootstrap complete!"
    echo ""
    echo "Next steps:"
    echo "1. cd $INFRA_DIR/environments/dev"
    echo "2. cp terraform.tfvars.example terraform.tfvars"
    echo "3. Edit terraform.tfvars with your values"
    echo "4. terraform init"
    echo "5. terraform plan"
    echo "6. terraform apply"
}

main
