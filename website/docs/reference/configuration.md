# Configuration

roea-ai can be configured through configuration files, environment variables, and command-line arguments.

## Configuration File

The main configuration file is located at:

| Platform | Path |
|----------|------|
| Linux | `~/.config/roea-ai/config.toml` |
| macOS | `~/Library/Application Support/roea-ai/config.toml` |
| Windows | `%APPDATA%\roea-ai\config.toml` |

### Example Configuration

```toml
# roea-ai configuration

[daemon]
# gRPC server address
listen_addr = "[::1]:50051"
# Log level: trace, debug, info, warn, error
log_level = "info"

[monitoring]
# Process polling interval in milliseconds
poll_interval_ms = 100
# Enable eBPF monitoring on Linux (if available)
enable_ebpf = true
# Maximum processes to track
max_processes = 10000

[storage]
# Database file path (empty for default)
database_path = ""
# Data retention in days
retention_days = 7
# Maximum database size in MB
max_size_mb = 1000

[network]
# Network polling interval in milliseconds
poll_interval_ms = 500
# Enable Unix socket tracking
track_unix_sockets = true

[file]
# File monitoring poll interval in milliseconds
poll_interval_ms = 1000
# Paths to ignore (glob patterns)
ignore_patterns = [
    "/proc/*",
    "/sys/*",
    "/dev/*",
    "*.swp",
    "*.pyc",
    "__pycache__/*",
    "node_modules/*",
    ".git/objects/*"
]

[telemetry]
# Enable OpenTelemetry export
enabled = false
# OTLP endpoint
endpoint = "http://localhost:4317"
# Service name for telemetry
service_name = "roea-ai"

[sentry]
# Sentry DSN for error reporting (empty to disable)
dsn = ""
# Environment name
environment = "production"
```

## Environment Variables

Environment variables override configuration file settings:

| Variable | Description | Default |
|----------|-------------|---------|
| `ROEA_LISTEN_ADDR` | gRPC server address | `[::1]:50051` |
| `ROEA_LOG_LEVEL` | Log verbosity | `info` |
| `ROEA_DATABASE_PATH` | Database file location | (platform default) |
| `SENTRY_DSN` | Sentry error tracking DSN | (disabled) |
| `SENTRY_ENVIRONMENT` | Sentry environment | `development` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry endpoint | (disabled) |

## Command-Line Arguments

```bash
roea-agent [OPTIONS]

Options:
  -c, --config <FILE>     Configuration file path
  -l, --listen <ADDR>     gRPC listen address
  -v, --verbose           Increase log verbosity
  -q, --quiet             Decrease log verbosity
  --version               Print version and exit
  -h, --help              Print help
```

## Agent Signatures

Agent signatures define how roea-ai detects AI coding agents. They're stored in:

| Platform | Path |
|----------|------|
| Linux | `~/.config/roea-ai/signatures/` |
| macOS | `~/Library/Application Support/roea-ai/signatures/` |
| Windows | `%APPDATA%\roea-ai\signatures\` |

### Signature Format

```yaml
# claude_code.yaml
name: claude-code
display_name: Claude Code
icon: claude.png

# Process detection
process_names:
  - claude
  - claude-code

# Command line patterns (regex)
cmdline_patterns:
  - "claude\\s+(chat|code|api)"
  - "--api-key"

# Executable paths (glob patterns)
exe_patterns:
  - "**/claude"
  - "**/claude-code"
  - "*/Claude*"

# Network endpoints this agent uses
endpoints:
  - api.anthropic.com
  - sentry.io

# Track children of detected processes
track_children: true

# Parent process hints (for detecting via parent)
parent_hints:
  - vscode
  - cursor
```

### Built-in Signatures

roea-ai includes signatures for:

| Agent | File |
|-------|------|
| Claude Code | `claude_code.yaml` |
| Cursor | `cursor.yaml` |
| VS Code Copilot | `copilot.yaml` |
| Windsurf | `windsurf.yaml` |
| Aider | `aider.yaml` |
| Continue.dev | `continue.yaml` |
| Cline | `cline.yaml` |

### Custom Signatures

Create custom signatures for unsupported agents:

```yaml
# ~/.config/roea-ai/signatures/my_agent.yaml
name: my-custom-agent
display_name: My Custom Agent
process_names:
  - my-agent
  - my-agent-helper
cmdline_patterns:
  - "my-agent.*--start"
track_children: true
```

Restart roea-agent to load new signatures.

## Storage Configuration

### Database Location

Default database locations:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/roea-ai/roea.db` |
| macOS | `~/Library/Application Support/roea-ai/roea.db` |
| Windows | `%LOCALAPPDATA%\roea-ai\roea.db` |

### Data Retention

Configure how long data is kept:

```toml
[storage]
retention_days = 7  # Keep 7 days of history
max_size_mb = 1000  # Maximum 1GB database size
```

### Vacuum Schedule

DuckDB automatically vacuums the database. Force a vacuum:

```bash
roea-agent --vacuum
```

## Monitoring Configuration

### Process Monitoring

```toml
[monitoring]
# Lower = more responsive, higher = less CPU
poll_interval_ms = 100

# Disable if you don't need complete capture
enable_ebpf = true

# Limit memory usage
max_processes = 10000
```

### Network Monitoring

```toml
[network]
poll_interval_ms = 500

# Disable if not needed
track_unix_sockets = true

# Custom endpoint classifications
[network.endpoints]
"api.mycompany.com" = "internal"
"*.mycompany.com" = "internal"
```

### File Monitoring

```toml
[file]
poll_interval_ms = 1000

# Ignore noisy paths
ignore_patterns = [
    "node_modules/*",
    ".git/objects/*",
    "*.log",
    "/tmp/*"
]
```

## Telemetry Export

### OpenTelemetry

Export data to any OTLP-compatible backend:

```toml
[telemetry]
enabled = true
endpoint = "http://localhost:4317"
service_name = "roea-ai"
```

Supported backends:
- Jaeger
- Zipkin
- Honeycomb
- Datadog
- New Relic
- Grafana Tempo

### Sentry

Error tracking with Sentry:

```toml
[sentry]
dsn = "https://key@sentry.io/project"
environment = "production"
sample_rate = 1.0
traces_sample_rate = 0.1
```

## UI Configuration

The UI stores preferences in:

| Platform | Path |
|----------|------|
| Linux | `~/.config/roea-ai/ui.json` |
| macOS | `~/Library/Application Support/roea-ai/ui.json` |
| Windows | `%APPDATA%\roea-ai\ui.json` |

```json
{
  "theme": "dark",
  "defaultLayout": "force",
  "showNetworkOverlay": true,
  "graphZoomLevel": 1.0,
  "sidebarWidth": 250,
  "detailsPanelWidth": 400
}
```

## Configuration Precedence

Settings are applied in this order (later overrides earlier):

1. Built-in defaults
2. Configuration file (`config.toml`)
3. Environment variables
4. Command-line arguments

## Troubleshooting

### Configuration Not Loading

1. Check file path and permissions
2. Validate TOML syntax: `toml-lint config.toml`
3. Check logs for parsing errors

### Signatures Not Detected

1. Verify YAML syntax
2. Check file is in signatures directory
3. Restart roea-agent after adding

### Environment Variables Ignored

1. Ensure proper shell export
2. Check variable name spelling
3. Restart daemon after setting

## Related

- [Linux eBPF Setup](/reference/ebpf)
- [OpenTelemetry Export](/reference/opentelemetry)
- [Troubleshooting](/reference/troubleshooting)
