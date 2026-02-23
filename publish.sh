#!/usr/bin/env bash
set -euo pipefail

#
# Environment variables:
#   CARGO_REGISTRY_TOKEN  - crates.io API token (required unless DRY_RUN=1)
#   DRY_RUN               - If set to 1, run cargo publish --dry-run (default: 0)
#   SKIP_BUILD_CHECK      - If set to 1, skip the release build check (default: 0)
#   PUBLISH_CRATES        - Space-separated list of crates to publish (default: "tuai-common tuai")
#   PUBLISH_DELAY         - Seconds to wait between crate publishes (default: 30)

DRY_RUN="${DRY_RUN:-0}"
SKIP_BUILD_CHECK="${SKIP_BUILD_CHECK:-0}"
PUBLISH_CRATES="${PUBLISH_CRATES:-tuai-common tuai}"
PUBLISH_DELAY="${PUBLISH_DELAY:-30}"

if [[ "$DRY_RUN" != "1" && -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
    echo "Error: CARGO_REGISTRY_TOKEN is required for non-dry-run publishes"
    echo "  Get a token at https://crates.io/settings/tokens"
    echo "  Or set DRY_RUN=1 to test without publishing"
    exit 1
fi

echo "==> Publishing tuai crates"
echo "    Crates: $PUBLISH_CRATES"
echo "    Dry run: $DRY_RUN"

if [[ "$SKIP_BUILD_CHECK" != "1" ]]; then
    echo "==> Building release to verify..."
    cargo build --release
fi

published=0
for crate in $PUBLISH_CRATES; do
    if [[ $published -gt 0 && "$DRY_RUN" != "1" ]]; then
        echo "==> Waiting ${PUBLISH_DELAY}s for crates.io index to update..."
        sleep "$PUBLISH_DELAY"
    fi

    echo "==> Publishing $crate..."
    args=(cargo publish -p "$crate")
    if [[ "$DRY_RUN" == "1" ]]; then
        args+=(--dry-run)
    fi
    "${args[@]}"
    published=$((published + 1))
done

echo "==> Done"
