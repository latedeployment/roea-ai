# Contributing to roea-ai

Thank you for your interest in contributing to roea-ai! This guide will help you get started.

## Ways to Contribute

### Report Bugs

Found a bug? Please [open an issue](https://github.com/your-org/roea-ai/issues/new) with:
- roea-ai version (`roea-agent --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or screenshots

### Suggest Features

Have an idea? Open a [feature request](https://github.com/your-org/roea-ai/issues/new?template=feature_request.md) describing:
- The problem you're trying to solve
- Your proposed solution
- Alternative approaches you considered

### Improve Documentation

Documentation improvements are always welcome:
- Fix typos or unclear explanations
- Add examples
- Improve tutorials
- Translate to other languages

### Submit Code

Ready to code? Here's how to get started.

## Development Setup

### Prerequisites

- **Rust 1.75+** with cargo
- **Node.js 18+** with npm
- **protobuf-compiler** for gRPC
- **clang/llvm** (Linux, for eBPF)

### Clone and Build

```bash
# Clone repository
git clone https://github.com/your-org/roea-ai.git
cd roea-ai

# Install Rust dependencies
cargo build

# Install UI dependencies
cd crates/roea-ui && npm install

# Run tests
cargo test

# Run the agent
cargo run --bin roea-agent

# Run the UI in dev mode
cd crates/roea-ui && npm run tauri dev
```

### Project Structure

```
roea-ai/
├── Cargo.toml                # Workspace manifest
├── proto/                    # gRPC definitions
├── signatures/               # Agent signatures
└── crates/
    ├── roea-common/          # Shared types
    ├── roea-agent/           # Monitoring daemon
    │   ├── src/
    │   │   ├── monitor/      # Process monitoring
    │   │   ├── network/      # Network tracking
    │   │   ├── file/         # File monitoring
    │   │   ├── storage/      # DuckDB storage
    │   │   └── grpc/         # gRPC server
    │   └── tests/
    └── roea-ui/              # Desktop application
        ├── src/              # React frontend
        └── src-tauri/        # Tauri backend
```

## Coding Guidelines

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Write tests for new functionality
- Document public APIs with doc comments

```rust
/// Monitors process events from the kernel.
///
/// # Examples
///
/// ```
/// let monitor = ProcessMonitor::new()?;
/// monitor.start().await?;
/// ```
pub struct ProcessMonitor { /* ... */ }
```

### TypeScript Code

- Use TypeScript for all frontend code
- Follow existing code style
- Run `npm run lint` before committing
- Write component tests with Playwright

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for Cline agent detection
fix: handle process exit race condition
docs: improve eBPF setup instructions
test: add network monitor unit tests
chore: update dependencies
```

## Pull Request Process

1. **Fork the repository** and create a branch:
   ```bash
   git checkout -b feat/my-feature
   ```

2. **Make your changes** following the coding guidelines

3. **Test your changes**:
   ```bash
   cargo test
   cargo clippy
   cd crates/roea-ui && npm run test
   ```

4. **Commit with a descriptive message**

5. **Push to your fork**:
   ```bash
   git push origin feat/my-feature
   ```

6. **Open a Pull Request** with:
   - Description of changes
   - Related issue number (if any)
   - Screenshots for UI changes

7. **Address review feedback**

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test monitor::tests::test_process_tree

# Run with logging
RUST_LOG=debug cargo test
```

### Integration Tests

```bash
# Run E2E tests
cd crates/roea-ui && npm run test:e2e
```

### Benchmarks

```bash
# Run benchmarks
cargo bench
```

## Adding Agent Signatures

To add support for a new AI agent:

1. Create a signature file in `signatures/`:
   ```yaml
   # signatures/new-agent.yaml
   name: new-agent
   display_name: New Agent
   process_patterns:
     - "new-agent"
   command_patterns:
     - "--new-agent-flag"
   network_endpoints:
     - "api.new-agent.com"
   track_children: true
   ```

2. Add tests for the signature

3. Update documentation

4. Submit PR

## Code of Conduct

Please be respectful and constructive in all interactions. We follow the [Contributor Covenant](https://www.contributor-covenant.org/).

## Questions?

- Open a [Discussion](https://github.com/your-org/roea-ai/discussions)
- Join our community chat (coming soon)
- Email: contributors@roea.ai

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
