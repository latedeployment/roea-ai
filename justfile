# roea-ai justfile
# Run `just` or `just help` to see available commands

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
    @echo "{{cyan}}roea-ai{{reset}} - Observability for AI Coding Agents"
    @echo ""
    @echo "{{green}}Usage:{{reset}}"
    @echo "  just {{yellow}}<recipe>{{reset}}"
    @echo ""
    @echo "{{green}}Quick Start:{{reset}}"
    @echo "  just setup      # Install all dependencies"
    @echo "  just dev        # Run agent + UI in development mode"
    @echo ""
    @echo "{{green}}Available recipes:{{reset}}"
    @just --list --unsorted

#---------------------------------------------------------------------------
# Setup & Dependencies
#---------------------------------------------------------------------------

# Full setup: install all dependencies
setup: install-deps
    @echo "{{green}}Setup complete!{{reset}}"
    @echo "Run 'just dev' to start developing"

# Install Rust and Node.js dependencies
install-deps:
    @echo "{{cyan}}Installing Rust dependencies...{{reset}}"
    cargo fetch
    @echo "{{cyan}}Installing UI dependencies...{{reset}}"
    cd crates/roea-ui && npm install
    @echo "{{cyan}}Installing Playwright browsers...{{reset}}"
    cd crates/roea-ui && npx playwright install chromium

# Install all dependencies including all browsers
install-deps-full: install-deps
    cd crates/roea-ui && npx playwright install

#---------------------------------------------------------------------------
# Building
#---------------------------------------------------------------------------

# Build in debug mode
build:
    @echo "{{cyan}}Building roea-agent (debug)...{{reset}}"
    cargo build
    @echo "{{cyan}}Building roea-ui (debug)...{{reset}}"
    cd crates/roea-ui && npm run build

# Build in release mode
build-release:
    @echo "{{cyan}}Building roea-agent (release)...{{reset}}"
    cargo build --release
    @echo "{{cyan}}Building roea-ui (release)...{{reset}}"
    cd crates/roea-ui && npm run build

# Build only the agent
build-agent:
    cargo build --release --bin roea-agent

# Build only the UI
build-ui:
    cd crates/roea-ui && npm run tauri build

#---------------------------------------------------------------------------
# Running
#---------------------------------------------------------------------------

# Run agent and UI in development mode (parallel)
dev:
    @echo "{{cyan}}Starting roea-ai in development mode...{{reset}}"
    @echo "{{yellow}}Note: Run in two terminals or use 'just run-agent' and 'just run-ui' separately{{reset}}"
    @echo ""
    @echo "Terminal 1: just run-agent"
    @echo "Terminal 2: just run-ui"

# Run the monitoring agent
run-agent:
    @echo "{{cyan}}Starting roea-agent...{{reset}}"
    cargo run --bin roea-agent

# Run the agent in release mode
run-agent-release:
    @echo "{{cyan}}Starting roea-agent (release)...{{reset}}"
    cargo run --release --bin roea-agent

# Run the UI in development mode
run-ui:
    @echo "{{cyan}}Starting roea-ui...{{reset}}"
    cd crates/roea-ui && npm run tauri dev

# Run only the web UI (without Tauri)
run-ui-web:
    @echo "{{cyan}}Starting web UI...{{reset}}"
    cd crates/roea-ui && npm run dev

#---------------------------------------------------------------------------
# Testing
#---------------------------------------------------------------------------

# Run all tests
test: test-rust test-ui

# Run Rust unit tests
test-rust:
    @echo "{{cyan}}Running Rust tests...{{reset}}"
    cargo test --workspace

# Run Rust tests with output
test-rust-verbose:
    cargo test --workspace -- --nocapture

# Run UI E2E tests
test-ui:
    @echo "{{cyan}}Running Playwright tests...{{reset}}"
    cd crates/roea-ui && npm run test:e2e

# Run UI tests with visible browser
test-ui-headed:
    cd crates/roea-ui && npm run test:e2e -- --headed

# Run UI tests in debug mode
test-ui-debug:
    cd crates/roea-ui && npm run test:e2e -- --debug

# Run benchmarks
bench:
    @echo "{{cyan}}Running benchmarks...{{reset}}"
    cargo bench

#---------------------------------------------------------------------------
# Code Quality
#---------------------------------------------------------------------------

# Check code compiles without building
check:
    cargo check --workspace

# Run all linters
lint:
    @echo "{{cyan}}Running Clippy...{{reset}}"
    cargo clippy --workspace -- -D warnings
    @echo "{{cyan}}Running ESLint...{{reset}}"
    cd crates/roea-ui && npm run lint 2>/dev/null || true

# Format all code
format:
    @echo "{{cyan}}Formatting Rust code...{{reset}}"
    cargo fmt
    @echo "{{cyan}}Formatting TypeScript code...{{reset}}"
    cd crates/roea-ui && npm run format 2>/dev/null || npx prettier --write "src/**/*.{ts,tsx}"

# Check formatting without changes
format-check:
    cargo fmt -- --check
    cd crates/roea-ui && npx prettier --check "src/**/*.{ts,tsx}" 2>/dev/null || true

#---------------------------------------------------------------------------
# Documentation
#---------------------------------------------------------------------------

# Build documentation
docs:
    @echo "{{cyan}}Building Rust docs...{{reset}}"
    cargo doc --no-deps
    @echo "{{cyan}}Building website docs...{{reset}}"
    cd website && npm install && npm run build

# Serve documentation locally
docs-serve:
    cd website && npm run dev

#---------------------------------------------------------------------------
# Cleaning
#---------------------------------------------------------------------------

# Clean all build artifacts
clean:
    @echo "{{cyan}}Cleaning Rust artifacts...{{reset}}"
    cargo clean
    @echo "{{cyan}}Cleaning UI artifacts...{{reset}}"
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
    @echo "{{cyan}}Starting roea-agent with eBPF (sudo)...{{reset}}"
    sudo cargo run --release --bin roea-agent

#---------------------------------------------------------------------------
# Release
#---------------------------------------------------------------------------

# Build release artifacts for all platforms
release-build:
    @echo "{{cyan}}Building release artifacts...{{reset}}"
    cargo build --release
    cd crates/roea-ui && npm run tauri build

#---------------------------------------------------------------------------
# Utility
#---------------------------------------------------------------------------

# Show version information
version:
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"
    @echo "Node: $(node --version)"
    @echo "npm: $(npm --version)"

# Watch for changes and rebuild
watch:
    cargo watch -x check -x test

# Count lines of code
loc:
    @echo "{{cyan}}Lines of code:{{reset}}"
    @find crates -name "*.rs" | xargs wc -l | tail -1
    @find crates/roea-ui/src -name "*.ts" -o -name "*.tsx" | xargs wc -l 2>/dev/null | tail -1 || echo "0 TypeScript"
