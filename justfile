# roea-ai justfile
# Run `just` or `just help` to see available commands

set shell := ["bash", "-c"]

# Cargo env setup - source this in recipes that need cargo
cargo_env := "source $HOME/.cargo/env 2>/dev/null || true"

# Colors for output
cyan := '\033[36m'
green := '\033[32m'
yellow := '\033[33m'
reset := '\033[0m'

# Default recipe - show help
default: help

#---------------------------------------------------------------------------
# Help
#---------------------------------------------------------------------------

# Show this help message
help:
    @echo -e "{{cyan}}roea-ai{{reset}} - Observability for AI Coding Agents"
    @echo ""
    @echo -e "{{green}}Usage:{{reset}}"
    @echo -e "  just {{yellow}}<recipe>{{reset}}"
    @echo ""
    @echo -e "{{green}}Quick Start:{{reset}}"
    @echo "  just install-system-deps-fedora  # Install system deps (Fedora/RHEL)"
    @echo "  just install-system-deps-debian  # Install system deps (Debian/Ubuntu)"
    @echo "  just setup      # Install all dependencies"
    @echo "  just dev        # Run agent + UI in development mode"
    @echo ""
    @echo -e "{{green}}Available recipes:{{reset}}"
    @just --list --unsorted

#---------------------------------------------------------------------------
# Setup & Dependencies
#---------------------------------------------------------------------------

# Full setup: install all dependencies
setup: install-deps ebpf-setup
    @echo -e "{{green}}Setup complete!{{reset}}"
    @echo "Run 'just dev' to start developing"

# Install system dependencies (Fedora/RHEL)
install-system-deps-fedora:
    @echo -e "{{cyan}}Installing system dependencies (Fedora/RHEL)...{{reset}}"
    sudo dnf install -y gtk3-devel openssl-devel webkit2gtk4.1-devel javascriptcoregtk4.1-devel libsoup3-devel elfutils-libelf-devel clang gcc-c++ protobuf-compiler bpftool

# Install system dependencies (Debian/Ubuntu)
install-system-deps-debian:
    @echo -e "{{cyan}}Installing system dependencies (Debian/Ubuntu)...{{reset}}"
    sudo apt-get install -y libgtk-3-dev libssl-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libelf-dev clang g++ protobuf-compiler linux-tools-common

# Install Rust and Node.js dependencies
install-deps:
    @echo -e "{{cyan}}Installing Rust dependencies...{{reset}}"
    {{cargo_env}} && cargo fetch
    @echo -e "{{cyan}}Installing UI dependencies...{{reset}}"
    cd crates/roea-ui && npm install
    @echo -e "{{cyan}}Installing Playwright browsers...{{reset}}"
    cd crates/roea-ui && npx playwright install chromium

# Install all dependencies including all browsers
install-deps-full: install-deps
    cd crates/roea-ui && npx playwright install

#---------------------------------------------------------------------------
# Building
#---------------------------------------------------------------------------

# Ensure UI dist placeholder exists for Tauri compile
_ensure-ui-dist:
    @mkdir -p crates/roea-ui/dist
    @[ -f crates/roea-ui/dist/index.html ] || echo '<!DOCTYPE html><html><head><title>Roea UI</title></head><body><div id="root"></div></body></html>' > crates/roea-ui/dist/index.html

# Build in debug mode
build: _ensure-ui-dist
    @echo -e "{{cyan}}Building roea-agent (debug)...{{reset}}"
    {{cargo_env}} && cargo build
    @echo -e "{{cyan}}Building roea-ui (debug)...{{reset}}"
    cd crates/roea-ui && npm run build

# Build in release mode
build-release: _ensure-ui-dist
    @echo -e "{{cyan}}Building roea-agent (release)...{{reset}}"
    {{cargo_env}} && cargo build --release
    @echo -e "{{cyan}}Building roea-ui (release)...{{reset}}"
    cd crates/roea-ui && npm run build

# Build only the agent
build-agent:
    {{cargo_env}} && cargo build --release --bin roea-agent

# Build only the UI
build-ui: _ensure-ui-dist
    cd crates/roea-ui && npm run tauri build

#---------------------------------------------------------------------------
# Running
#---------------------------------------------------------------------------

# Run agent and UI in development mode (parallel)
dev:
    @echo -e "{{cyan}}Starting roea-ai in development mode...{{reset}}"
    @echo -e "{{yellow}}Note: Run in two terminals or use 'just run-agent' and 'just run-ui' separately{{reset}}"
    @echo ""
    @echo "Terminal 1: just run-agent"
    @echo "Terminal 2: just run-ui"

# Run the monitoring agent
run-agent:
    @echo -e "{{cyan}}Starting roea-agent...{{reset}}"
    {{cargo_env}} && cargo run --bin roea-agent

# Run the agent in release mode
run-agent-release:
    @echo -e "{{cyan}}Starting roea-agent (release)...{{reset}}"
    {{cargo_env}} && cargo run --release --bin roea-agent

# Run the UI in development mode
run-ui: _ensure-ui-dist
    @echo -e "{{cyan}}Starting roea-ui...{{reset}}"
    cd crates/roea-ui && npm run tauri dev

# Run only the web UI (without Tauri)
run-ui-web:
    @echo -e "{{cyan}}Starting web UI...{{reset}}"
    cd crates/roea-ui && npm run dev

#---------------------------------------------------------------------------
# Testing
#---------------------------------------------------------------------------

# Run all tests
test: test-rust test-ui

# Run Rust unit tests
test-rust: _ensure-ui-dist
    @echo -e "{{cyan}}Running Rust tests...{{reset}}"
    {{cargo_env}} && cargo test --workspace

# Run Rust tests with output
test-rust-verbose: _ensure-ui-dist
    {{cargo_env}} && cargo test --workspace -- --nocapture

# Run UI E2E tests
test-ui:
    @echo -e "{{cyan}}Running Playwright tests...{{reset}}"
    cd crates/roea-ui && npm run test:e2e

# Run UI tests with visible browser
test-ui-headed:
    cd crates/roea-ui && npm run test:e2e -- --headed

# Run UI tests in debug mode
test-ui-debug:
    cd crates/roea-ui && npm run test:e2e -- --debug

# Run benchmarks
bench:
    @echo -e "{{cyan}}Running benchmarks...{{reset}}"
    {{cargo_env}} && cargo bench

#---------------------------------------------------------------------------
# Code Quality
#---------------------------------------------------------------------------

# Check code compiles without building
check: _ensure-ui-dist
    {{cargo_env}} && cargo check --workspace

# Run all linters
lint: _ensure-ui-dist
    @echo -e "{{cyan}}Running Clippy...{{reset}}"
    {{cargo_env}} && cargo clippy --workspace -- -D warnings
    @echo -e "{{cyan}}Running ESLint...{{reset}}"
    cd crates/roea-ui && npm run lint 2>/dev/null || true

# Format all code
format:
    @echo -e "{{cyan}}Formatting Rust code...{{reset}}"
    {{cargo_env}} && cargo fmt
    @echo -e "{{cyan}}Formatting TypeScript code...{{reset}}"
    cd crates/roea-ui && npm run format 2>/dev/null || npx prettier --write "src/**/*.{ts,tsx}"

# Check formatting without changes
format-check:
    {{cargo_env}} && cargo fmt -- --check
    cd crates/roea-ui && npx prettier --check "src/**/*.{ts,tsx}" 2>/dev/null || true

#---------------------------------------------------------------------------
# Documentation
#---------------------------------------------------------------------------

# Build documentation
docs: _ensure-ui-dist
    @echo -e "{{cyan}}Building Rust docs...{{reset}}"
    {{cargo_env}} && cargo doc --no-deps
    @echo -e "{{cyan}}Building website docs...{{reset}}"
    cd website && npm install && npm run build

# Serve documentation locally
docs-serve:
    cd website && npm run dev

#---------------------------------------------------------------------------
# Cleaning
#---------------------------------------------------------------------------

# Clean all build artifacts
clean:
    @echo -e "{{cyan}}Cleaning Rust artifacts...{{reset}}"
    {{cargo_env}} && cargo clean
    @echo -e "{{cyan}}Cleaning UI artifacts...{{reset}}"
    rm -rf crates/roea-ui/dist
    rm -rf crates/roea-ui/node_modules/.vite
    rm -rf crates/roea-ui/src-tauri/target
    rm -rf crates/roea-ui/test-results

# Clean everything including node_modules
clean-all: clean
    rm -rf crates/roea-ui/node_modules
    rm -rf website/node_modules

#---------------------------------------------------------------------------
# eBPF (Linux only)
#---------------------------------------------------------------------------

# Setup eBPF development (Linux only)
ebpf-setup:
    #!/usr/bin/env bash
    echo -e "{{cyan}}Generating vmlinux.h from kernel BTF...{{reset}}"
    if [ -f /sys/kernel/btf/vmlinux ]; then
        bpftool btf dump file /sys/kernel/btf/vmlinux format c > crates/roea-agent/src/bpf/vmlinux.h
        echo -e "{{green}}vmlinux.h generated successfully{{reset}}"
    else
        echo -e "{{yellow}}BTF not available. eBPF monitoring will be disabled.{{reset}}"
    fi

# Run agent with eBPF (requires sudo)
run-agent-ebpf:
    @echo -e "{{cyan}}Starting roea-agent with eBPF (sudo)...{{reset}}"
    {{cargo_env}} && sudo -E cargo run --release --bin roea-agent

#---------------------------------------------------------------------------
# Release
#---------------------------------------------------------------------------

# Build release artifacts for all platforms
release-build: _ensure-ui-dist
    @echo -e "{{cyan}}Building release artifacts...{{reset}}"
    {{cargo_env}} && cargo build --release
    cd crates/roea-ui && npm run tauri build

#---------------------------------------------------------------------------
# Utility
#---------------------------------------------------------------------------

# Show version information
version:
    @echo "Rust: $({{cargo_env}} && rustc --version)"
    @echo "Cargo: $({{cargo_env}} && cargo --version)"
    @echo "Node: $(node --version)"
    @echo "npm: $(npm --version)"

# Watch for changes and rebuild
watch:
    {{cargo_env}} && cargo watch -x check -x test

# Count lines of code
loc:
    @echo -e "{{cyan}}Lines of code:{{reset}}"
    @find crates -name "*.rs" | xargs wc -l | tail -1
    @find crates/roea-ui/src -name "*.ts" -o -name "*.tsx" | xargs wc -l 2>/dev/null | tail -1 || echo "0 TypeScript"
