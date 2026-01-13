# Environment Variables

Complete reference for roea-ai environment variables.

## Core Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ROEA_CONFIG` | Path to config file | `~/.config/roea/roea.toml` |
| `ROEA_DATA_DIR` | Data directory | Platform-specific |
| `ROEA_LOG_LEVEL` | Log verbosity | `info` |
| `ROEA_LOG_FILE` | Log file path | None (stderr) |

## Agent Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `ROEA_SIGNATURES_PATH` | Colon-separated signature paths | Built-in |
| `ROEA_DISABLE_AGENTS` | Comma-separated agents to ignore | None |
| `ROEA_CUSTOM_AGENTS` | Path to custom agent definitions | None |

## Monitoring Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `ROEA_PROCESS_POLL_MS` | Process poll interval (ms) | `1000` |
| `ROEA_NETWORK_POLL_MS` | Network poll interval (ms) | `2000` |
| `ROEA_FILE_POLL_MS` | File monitor poll interval (ms) | `1000` |
| `ROEA_EBPF_ENABLED` | Enable eBPF monitoring | `true` |
| `ROEA_EBPF_FALLBACK` | Fall back to polling if eBPF fails | `true` |

## Storage Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `ROEA_DB_PATH` | Database file path | `data.duckdb` |
| `ROEA_DB_MAX_SIZE` | Maximum database size | Unlimited |
| `ROEA_RETENTION_DAYS` | Data retention in days | `30` |

## Network Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `ROEA_GRPC_ADDR` | gRPC server address | `127.0.0.1:50051` |
| `ROEA_GRPC_TLS` | Enable TLS for gRPC | `false` |
| `ROEA_GRPC_CERT` | TLS certificate path | None |
| `ROEA_GRPC_KEY` | TLS key path | None |

## UI Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `ROEA_UI_PORT` | Web UI port (if enabled) | `8080` |
| `ROEA_UI_HOST` | Web UI host | `127.0.0.1` |

## Telemetry & Observability

| Variable | Description | Default |
|----------|-------------|---------|
| `SENTRY_DSN` | Sentry error tracking DSN | None |
| `SENTRY_ENVIRONMENT` | Sentry environment tag | `production` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP endpoint | None |
| `OTEL_SERVICE_NAME` | Service name for traces | `roea-agent` |

## Debug Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Rust logging config | `roea=info` |
| `RUST_BACKTRACE` | Enable backtraces | `0` |
| `ROEA_DEBUG` | Enable debug mode | `false` |
| `ROEA_PROFILE` | Enable profiling | `false` |

## Examples

### Basic Setup

```bash
export ROEA_LOG_LEVEL=debug
export ROEA_DATA_DIR=~/.roea
export ROEA_PROCESS_POLL_MS=500
```

### Production Setup

```bash
export ROEA_LOG_LEVEL=warn
export ROEA_LOG_FILE=/var/log/roea/agent.log
export ROEA_DB_MAX_SIZE=10GB
export ROEA_RETENTION_DAYS=90
export SENTRY_DSN=https://xxx@sentry.io/xxx
```

### Development Setup

```bash
export RUST_LOG=roea=trace,roea_agent::monitor=debug
export RUST_BACKTRACE=1
export ROEA_DEBUG=true
export ROEA_EBPF_ENABLED=false  # Easier debugging
```

### Custom Signatures

```bash
export ROEA_SIGNATURES_PATH=/etc/roea/signatures:/home/user/.roea/signatures
export ROEA_CUSTOM_AGENTS=/home/user/.roea/my-agents.yaml
```

## Precedence

Configuration is loaded in this order (later overrides earlier):

1. Built-in defaults
2. System config (`/etc/roea/roea.toml`)
3. User config (`~/.config/roea/roea.toml`)
4. `ROEA_CONFIG` file
5. Environment variables
6. Command-line flags

## See Also

- [Configuration](/reference/configuration) - File-based configuration
- [Storage Settings](/reference/storage) - Database configuration
- [Linux eBPF Setup](/reference/ebpf) - eBPF configuration
