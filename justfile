# tuai - TUI for AI agent monitoring

set shell := ["bash", "-c"]

cargo_env := "source $HOME/.cargo/env 2>/dev/null || true"

default: help

help:
    @echo "tuai - like k9s, but for AI coding agents"
    @echo ""
    @echo "Usage: just <recipe>"
    @echo ""
    @just --list --unsorted

# Build debug
build: ebpf-setup
    {{cargo_env}} && cargo build

# Build release
build-release: ebpf-setup
    {{cargo_env}} && cargo build --release

# Run tuai (TUI mode, default)
run *ARGS:
    {{cargo_env}} && cargo run --bin tuai -- {{ARGS}}

# Run in release mode
run-release *ARGS:
    {{cargo_env}} && cargo run --release --bin tuai -- {{ARGS}}

# Run with eBPF (requires sudo, Linux only)
run-ebpf *ARGS:
    {{cargo_env}} && sudo -E cargo run --release --bin tuai -- {{ARGS}}

# Run tests
test:
    {{cargo_env}} && cargo test --workspace

# Run tests with output
test-verbose:
    {{cargo_env}} && cargo test --workspace -- --nocapture

# Check compilation
check:
    {{cargo_env}} && cargo check --workspace

# Run clippy
lint:
    {{cargo_env}} && cargo clippy --workspace -- -D warnings

# Format code
fmt:
    {{cargo_env}} && cargo fmt

# Check formatting
fmt-check:
    {{cargo_env}} && cargo fmt -- --check

# Clean build artifacts
clean:
    {{cargo_env}} && cargo clean

# Generate vmlinux.h for eBPF (Linux only)
ebpf-setup:
    #!/usr/bin/env bash
    if [ -f /sys/kernel/btf/vmlinux ]; then
        bpftool btf dump file /sys/kernel/btf/vmlinux format c > crates/tuai/src/bpf/vmlinux.h
        echo "vmlinux.h generated"
    else
        echo "BTF not available - eBPF will be disabled"
    fi

# Install system deps (Fedora)
deps-fedora:
    sudo dnf install -y elfutils-libelf-devel clang gcc-c++ protobuf-compiler bpftool

# Install system deps (Debian/Ubuntu)
deps-debian:
    sudo apt-get install -y libelf-dev clang g++ protobuf-compiler linux-tools-common

# Lines of code
loc:
    @find crates -name "*.rs" | xargs wc -l | tail -1
