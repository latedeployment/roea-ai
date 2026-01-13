# roea-ai Makefile
# Run `make help` to see available commands

.PHONY: help setup build build-release test test-rust test-ui lint format clean run run-agent run-ui dev install-deps check

# Default target
.DEFAULT_GOAL := help

# Colors for output
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

#---------------------------------------------------------------------------
# Help
#---------------------------------------------------------------------------

help: ## Show this help message
	@echo "$(CYAN)roea-ai$(RESET) - Observability for AI Coding Agents"
	@echo ""
	@echo "$(GREEN)Usage:$(RESET)"
	@echo "  make $(YELLOW)<target>$(RESET)"
	@echo ""
	@echo "$(GREEN)Quick Start:$(RESET)"
	@echo "  make setup      # Install all dependencies"
	@echo "  make dev        # Run agent + UI in development mode"
	@echo ""
	@echo "$(GREEN)Available targets:$(RESET)"
	@awk 'BEGIN {FS = ":.*##"; printf ""} /^[a-zA-Z_-]+:.*?##/ { printf "  $(YELLOW)%-15s$(RESET) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

#---------------------------------------------------------------------------
# Setup & Dependencies
#---------------------------------------------------------------------------

setup: install-deps ## Full setup: install all dependencies
	@echo "$(GREEN)Setup complete!$(RESET)"
	@echo "Run 'make dev' to start developing"

install-deps: ## Install Rust and Node.js dependencies
	@echo "$(CYAN)Installing Rust dependencies...$(RESET)"
	cargo fetch
	@echo "$(CYAN)Installing UI dependencies...$(RESET)"
	cd crates/roea-ui && npm install
	@echo "$(CYAN)Installing Playwright browsers...$(RESET)"
	cd crates/roea-ui && npx playwright install chromium

install-deps-full: install-deps ## Install all dependencies including all browsers
	cd crates/roea-ui && npx playwright install

#---------------------------------------------------------------------------
# Building
#---------------------------------------------------------------------------

build: ## Build in debug mode
	@echo "$(CYAN)Building roea-agent (debug)...$(RESET)"
	cargo build
	@echo "$(CYAN)Building roea-ui (debug)...$(RESET)"
	cd crates/roea-ui && npm run build

build-release: ## Build in release mode
	@echo "$(CYAN)Building roea-agent (release)...$(RESET)"
	cargo build --release
	@echo "$(CYAN)Building roea-ui (release)...$(RESET)"
	cd crates/roea-ui && npm run build

build-agent: ## Build only the agent
	cargo build --release --bin roea-agent

build-ui: ## Build only the UI
	cd crates/roea-ui && npm run tauri build

#---------------------------------------------------------------------------
# Running
#---------------------------------------------------------------------------

dev: ## Run agent and UI in development mode (parallel)
	@echo "$(CYAN)Starting roea-ai in development mode...$(RESET)"
	@echo "$(YELLOW)Note: Run in two terminals or use 'make run-agent' and 'make run-ui' separately$(RESET)"
	@echo ""
	@echo "Terminal 1: make run-agent"
	@echo "Terminal 2: make run-ui"

run-agent: ## Run the monitoring agent
	@echo "$(CYAN)Starting roea-agent...$(RESET)"
	cargo run --bin roea-agent

run-agent-release: ## Run the agent in release mode
	@echo "$(CYAN)Starting roea-agent (release)...$(RESET)"
	cargo run --release --bin roea-agent

run-ui: ## Run the UI in development mode
	@echo "$(CYAN)Starting roea-ui...$(RESET)"
	cd crates/roea-ui && npm run tauri dev

run-ui-web: ## Run only the web UI (without Tauri)
	@echo "$(CYAN)Starting web UI...$(RESET)"
	cd crates/roea-ui && npm run dev

#---------------------------------------------------------------------------
# Testing
#---------------------------------------------------------------------------

test: test-rust test-ui ## Run all tests

test-rust: ## Run Rust unit tests
	@echo "$(CYAN)Running Rust tests...$(RESET)"
	cargo test --workspace

test-rust-verbose: ## Run Rust tests with output
	cargo test --workspace -- --nocapture

test-ui: ## Run UI E2E tests
	@echo "$(CYAN)Running Playwright tests...$(RESET)"
	cd crates/roea-ui && npm run test:e2e

test-ui-headed: ## Run UI tests with visible browser
	cd crates/roea-ui && npm run test:e2e -- --headed

test-ui-debug: ## Run UI tests in debug mode
	cd crates/roea-ui && npm run test:e2e -- --debug

bench: ## Run benchmarks
	@echo "$(CYAN)Running benchmarks...$(RESET)"
	cargo bench

#---------------------------------------------------------------------------
# Code Quality
#---------------------------------------------------------------------------

check: ## Check code compiles without building
	cargo check --workspace

lint: ## Run all linters
	@echo "$(CYAN)Running Clippy...$(RESET)"
	cargo clippy --workspace -- -D warnings
	@echo "$(CYAN)Running ESLint...$(RESET)"
	cd crates/roea-ui && npm run lint 2>/dev/null || true

format: ## Format all code
	@echo "$(CYAN)Formatting Rust code...$(RESET)"
	cargo fmt
	@echo "$(CYAN)Formatting TypeScript code...$(RESET)"
	cd crates/roea-ui && npm run format 2>/dev/null || npx prettier --write "src/**/*.{ts,tsx}"

format-check: ## Check formatting without changes
	cargo fmt -- --check
	cd crates/roea-ui && npx prettier --check "src/**/*.{ts,tsx}" 2>/dev/null || true

#---------------------------------------------------------------------------
# Documentation
#---------------------------------------------------------------------------

docs: ## Build documentation
	@echo "$(CYAN)Building Rust docs...$(RESET)"
	cargo doc --no-deps
	@echo "$(CYAN)Building website docs...$(RESET)"
	cd website && npm install && npm run build

docs-serve: ## Serve documentation locally
	cd website && npm run dev

#---------------------------------------------------------------------------
# Cleaning
#---------------------------------------------------------------------------

clean: ## Clean all build artifacts
	@echo "$(CYAN)Cleaning Rust artifacts...$(RESET)"
	cargo clean
	@echo "$(CYAN)Cleaning UI artifacts...$(RESET)"
	rm -rf crates/roea-ui/dist
	rm -rf crates/roea-ui/node_modules/.vite
	rm -rf crates/roea-ui/src-tauri/target
	rm -rf crates/roea-ui/test-results

clean-all: clean ## Clean everything including node_modules
	rm -rf crates/roea-ui/node_modules
	rm -rf website/node_modules

#---------------------------------------------------------------------------
# eBPF (Linux only)
#---------------------------------------------------------------------------

ebpf-setup: ## Setup eBPF development (Linux only)
	@echo "$(CYAN)Generating vmlinux.h from kernel BTF...$(RESET)"
	@if [ -f /sys/kernel/btf/vmlinux ]; then \
		bpftool btf dump file /sys/kernel/btf/vmlinux format c > crates/roea-agent/src/bpf/vmlinux.h; \
		echo "$(GREEN)vmlinux.h generated successfully$(RESET)"; \
	else \
		echo "$(YELLOW)BTF not available. eBPF monitoring will be disabled.$(RESET)"; \
	fi

run-agent-ebpf: ## Run agent with eBPF (requires sudo)
	@echo "$(CYAN)Starting roea-agent with eBPF (sudo)...$(RESET)"
	sudo cargo run --release --bin roea-agent

#---------------------------------------------------------------------------
# Release
#---------------------------------------------------------------------------

release-build: ## Build release artifacts for all platforms
	@echo "$(CYAN)Building release artifacts...$(RESET)"
	cargo build --release
	cd crates/roea-ui && npm run tauri build

#---------------------------------------------------------------------------
# Utility
#---------------------------------------------------------------------------

version: ## Show version information
	@echo "Rust: $$(rustc --version)"
	@echo "Cargo: $$(cargo --version)"
	@echo "Node: $$(node --version)"
	@echo "npm: $$(npm --version)"

watch: ## Watch for changes and rebuild
	cargo watch -x check -x test

loc: ## Count lines of code
	@echo "$(CYAN)Lines of code:$(RESET)"
	@find crates -name "*.rs" | xargs wc -l | tail -1
	@find crates/roea-ui/src -name "*.ts" -o -name "*.tsx" | xargs wc -l 2>/dev/null | tail -1 || echo "0 TypeScript"
