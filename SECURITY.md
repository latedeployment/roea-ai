# Security Policy

## Overview

roea-ai is a local observability tool for AI coding agents. Security is a top priority given that the tool monitors system activity and may have access to sensitive process information.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in roea-ai, please report it responsibly:

1. **DO NOT** open a public GitHub issue for security vulnerabilities
2. Email security concerns to: [security@latedeploy.com]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes

We will respond within 48 hours and work with you to understand and address the issue.

## Security Measures

### Data Handling

- **No cloud transmission**: All data stays local by default
- **Sensitive data redaction**: API keys, tokens, and credentials are automatically redacted from logs
- **Path sanitization**: File paths are validated to prevent directory traversal attacks
- **Input validation**: All user inputs are validated before processing

### Permissions

- **Minimal privileges**: The agent runs with the minimum required permissions
- **No root requirement**: Standard operation does not require root/admin (except for eBPF on Linux)
- **Sandboxed UI**: The Tauri UI runs in a sandboxed webview

### Code Security

- **Dependency auditing**: All dependencies are scanned for known vulnerabilities
- **License compliance**: Only approved open-source licenses are allowed
- **SAST scanning**: Static analysis is run on every PR
- **Code review**: All changes require review before merging

### Build Security

- **Reproducible builds**: Build process is deterministic
- **Signed releases**: Release binaries are signed (when configured)
- **Checksum verification**: SHA256 checksums are provided for all releases
- **Supply chain protection**: cargo-deny enforces dependency policies

## Security Scanning

The following automated security checks run on every PR and nightly:

| Check | Tool | Description |
|-------|------|-------------|
| Rust vulnerabilities | cargo-audit | Checks for known CVEs in Rust dependencies |
| License compliance | cargo-deny | Ensures only approved licenses |
| Supply chain | cargo-deny | Validates dependency sources |
| NPM vulnerabilities | npm audit | Checks JavaScript dependencies |
| SAST | Clippy + ESLint | Static analysis for security patterns |
| Secret scanning | Custom | Detects accidentally committed secrets |
| CodeQL | GitHub | Deep semantic code analysis |

## Security Best Practices for Users

### Installation

1. Download only from official releases on GitHub
2. Verify checksums before installation:
   ```bash
   sha256sum -c checksums.txt
   ```
3. On macOS, allow only from identified developers in Security settings

### Configuration

1. Review and understand what processes will be monitored
2. Configure appropriate data retention policies
3. Use the UI over network access when possible

### Network Security

- The gRPC API listens on localhost only by default
- No external network connections are made unless configured
- TLS is recommended for any remote API access

## Architecture Security Considerations

### Process Monitoring

- Process information is read from OS APIs, not intercepted
- Command line arguments are captured and may contain sensitive data
- Use the security module's sanitization functions when logging

### Network Monitoring

- Network connections are enumerated, not intercepted
- No packet capture or deep packet inspection
- Only connection metadata (IPs, ports, states) is collected

### File Monitoring

- File paths are logged, not file contents
- Sensitive paths (credentials, keys) are flagged
- No file content is ever read or stored

### Storage

- Local DuckDB storage only
- Data is not encrypted at rest by default
- Retention policies can be configured to limit data lifetime

## Secure Development Guidelines

### For Contributors

1. **Never log sensitive data**: Use `security::sanitize_for_log()` for any user-provided strings
2. **Validate paths**: Use `security::is_safe_path()` before file operations
3. **Check environment variables**: Use `security::is_sensitive_env_var()` before logging env vars
4. **No unwrap in production**: Use proper error handling, not `.unwrap()`
5. **Minimize dependencies**: Each new dependency increases attack surface

### Code Patterns to Avoid

```rust
// BAD: Logging sensitive data
tracing::info!("API key: {}", api_key);

// GOOD: Sanitize before logging
tracing::info!("Config: {}", sanitize_for_log(&config_string));

// BAD: Unchecked path operations
std::fs::read(&user_provided_path)?;

// GOOD: Validate path first
if is_safe_path(&user_provided_path) {
    std::fs::read(&user_provided_path)?;
}

// BAD: Using unwrap
let value = result.unwrap();

// GOOD: Proper error handling
let value = result.context("Failed to get value")?;
```

### Security Lints

The following Clippy lints are enforced in security-critical code:

- `clippy::unwrap_used` - Deny
- `clippy::expect_used` - Deny
- `clippy::panic` - Warn
- `clippy::dbg_macro` - Warn
- `clippy::print_stdout` - Warn

## Third-Party Security

### Dependencies

All dependencies must:
- Be from trusted sources (crates.io, npm)
- Have acceptable licenses (MIT, Apache-2.0, BSD, etc.)
- Not have known critical vulnerabilities
- Be actively maintained

### Blocked Dependencies

The following dependency patterns are blocked:
- Crates from unknown registries
- Git dependencies from untrusted sources
- Yanked crate versions
- Crates with critical unpatched CVEs

## Incident Response

In case of a security incident:

1. **Containment**: Stop the affected roea-ai instance
2. **Assessment**: Check logs for any data exposure
3. **Notification**: Report to the security team
4. **Recovery**: Update to the latest patched version

## Changelog

| Date | Change |
|------|--------|
| 2026-01-13 | Initial security policy |
