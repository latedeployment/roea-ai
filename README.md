# roea-ai

**Observability for AI Coding Agents**

roea-ai is an open-source EDR-like tool that monitors what AI coding agents (Claude Code, Cursor, Copilot, Aider, etc.) are doing on your system. Track processes, network connections, and file access in real-time.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)

## Why roea-ai?

AI coding agents run with significant system access but offer limited visibility into their operations. roea-ai answers:

- What processes did the AI agent spawn?
- Which files did it read or modify?
- What network connections did it make?
- Which external APIs did it call?

## Features

- **Process Monitoring** - Real-time process tree tracking with parent-child relationships
- **Network Tracking** - TCP/UDP connections, API endpoint classification
- **File Access Monitoring** - Track file operations with noise filtering
- **Multi-Agent Support** - Monitor Claude Code, Cursor, Copilot, Aider, Windsurf, and more
- **Interactive Graph** - D3.js visualization of process trees and network connections
- **Search & Filter** - Query processes by name, PID, file path, or network endpoint
- **Local-First** - All data stays on your machine, no cloud dependencies
- **Cross-Platform** - Linux (with eBPF), macOS, and Windows support

## Screenshots

*Coming soon*

## Quick Start

```bash
# Clone the repository
git clone https://github.com/latedeployment/roea-ai.git
cd roea-ai

# Install dependencies and setup
make setup

# Run in development mode (two terminals)
make run-agent    # Terminal 1: Start the monitoring daemon
make run-ui       # Terminal 2: Start the desktop UI

# Or see all available commands
make help
```

## Installation

### Using Make (Recommended)

```bash
git clone https://github.com/latedeployment/roea-ai.git
cd roea-ai

make setup          # Install all dependencies
make build-release  # Build for production
make test           # Run all tests
```

### Manual Setup

```bash
# Prerequisites
# - Rust 1.75+
# - Node.js 18+
# - protobuf-compiler

# Clone
git clone https://github.com/latedeployment/roea-ai.git
cd roea-ai

# Build
cargo build --release

# Run the monitoring daemon
./target/release/roea-agent

# Run the desktop UI (in another terminal)
cd crates/roea-ui
npm install
npm run tauri dev
```

### Linux eBPF Setup (Optional)

For high-performance kernel-level monitoring on Linux:

```bash
# Check BTF support
ls /sys/kernel/btf/vmlinux

# Generate vmlinux.h
bpftool btf dump file /sys/kernel/btf/vmlinux format c > \
    crates/roea-agent/src/bpf/vmlinux.h

# Rebuild and run with privileges
cargo build --release
sudo ./target/release/roea-agent
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Desktop UI (Tauri)                      │
│                   React + D3.js + TypeScript                 │
└─────────────────────────────────────────────────────────────┘
                              │ gRPC
┌─────────────────────────────────────────────────────────────┐
│                     roea-agent (Rust)                        │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   Process    │   Network    │    File      │    Agent       │
│   Monitor    │   Monitor    │   Monitor    │   Signatures   │
├──────────────┴──────────────┴──────────────┴────────────────┤
│                    DuckDB Storage Layer                      │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
   ┌────▼────┐          ┌─────▼─────┐        ┌─────▼─────┐
   │  eBPF   │          │  sysinfo  │        │  osquery  │
   │ (Linux) │          │ (All OS)  │        │ (Optional)│
   └─────────┘          └───────────┘        └───────────┘
```

## Supported AI Agents

| Agent | Detection | Process Tree | Network | File Access |
|-------|-----------|--------------|---------|-------------|
| Claude Code | ✅ | ✅ | ✅ | ✅ |
| Cursor | ✅ | ✅ | ✅ | ✅ |
| VS Code + Copilot | ✅ | ✅ | ✅ | ✅ |
| Aider | ✅ | ✅ | ✅ | ✅ |
| Windsurf | ✅ | ✅ | ✅ | ✅ |
| Continue.dev | ✅ | ✅ | ✅ | ✅ |
| Cline | ✅ | ✅ | ✅ | ✅ |

Custom agents can be added via YAML signature files.

## Project Structure

```
roea-ai/
├── Cargo.toml              # Workspace manifest
├── proto/roea.proto        # gRPC service definitions
├── signatures/             # Agent detection signatures
├── crates/
│   ├── roea-common/        # Shared types and traits
│   ├── roea-agent/         # Monitoring daemon
│   │   └── src/
│   │       ├── monitor/    # Process monitoring
│   │       ├── network/    # Network tracking
│   │       ├── file/       # File monitoring
│   │       ├── storage/    # DuckDB storage
│   │       └── grpc/       # gRPC server
│   └── roea-ui/            # Desktop application
│       ├── src/            # React frontend
│       └── src-tauri/      # Tauri backend
├── infra/                  # Terraform IaC
├── website/                # Documentation site
└── project-status/         # Progress reports
```

## Development

### Prerequisites

- Rust 1.75+ with cargo
- Node.js 18+ with npm
- protobuf-compiler
- clang/llvm (Linux, for eBPF)

### Running Tests

```bash
# Rust unit tests
cargo test --workspace

# UI E2E tests
cd crates/roea-ui
npm install
npx playwright install
npm run test:e2e

# Benchmarks
cargo bench
```

### Building for Production

```bash
# Build agent
cargo build --release

# Build desktop app
cd crates/roea-ui
npm run tauri build
```

## Configuration

roea-ai uses TOML configuration:

```toml
# ~/.config/roea/config.toml

[monitor]
process_poll_interval = "1000ms"
network_poll_interval = "2000ms"

[storage]
path = "~/.local/share/roea/data.duckdb"
retention_days = 30

[grpc]
address = "127.0.0.1:50051"
```

## Adding Custom Agent Signatures

Create a YAML file in `~/.config/roea/signatures/`:

```yaml
# my-agent.yaml
name: my-agent
display_name: My Custom Agent
process_patterns:
  - "my-agent"
  - "my-agent-.*"
command_patterns:
  - "--my-agent-flag"
network_endpoints:
  - "api.my-agent.com"
track_children: true
```

## Security

- **Local-only** - No data leaves your machine
- **Read-only** - roea-ai only observes, never modifies
- **Sensitive data redaction** - API keys and tokens are filtered from logs
- **Open source** - Full code transparency

See [SECURITY.md](SECURITY.md) for our security policy.

## Roadmap

- [x] Core process monitoring
- [x] Network connection tracking
- [x] File access monitoring
- [x] Desktop UI with graph visualization
- [x] Linux eBPF support
- [x] CI/CD pipeline
- [ ] macOS Endpoint Security API
- [ ] Windows ETW integration
- [ ] Auto-updater
- [ ] Enterprise features

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](website/docs/contributing.md) for guidelines.

```bash
# Development setup
git clone https://github.com/latedeployment/roea-ai.git
cd roea-ai
cargo build
cd crates/roea-ui && npm install
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with:
- [Tauri](https://tauri.app/) - Desktop framework
- [DuckDB](https://duckdb.org/) - Embedded analytics database
- [D3.js](https://d3js.org/) - Data visualization
- [libbpf-rs](https://github.com/libbpf/libbpf-rs) - eBPF bindings for Rust
- [tonic](https://github.com/hyperium/tonic) - gRPC for Rust
