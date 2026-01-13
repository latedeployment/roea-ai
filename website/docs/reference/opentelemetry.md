# OpenTelemetry Export

roea-ai can export telemetry data to OpenTelemetry-compatible backends for centralized monitoring and analysis.

## Overview

OpenTelemetry integration enables:
- Export to any OTLP-compatible backend
- Traces for agent sessions
- Metrics for monitoring health
- Integration with existing observability stacks

## Configuration

Enable OpenTelemetry in `roea.toml`:

```toml
[telemetry.opentelemetry]
# Enable OTLP export
enabled = true

# OTLP endpoint
endpoint = "http://localhost:4317"

# Protocol: grpc or http
protocol = "grpc"

# Service name
service_name = "roea-agent"

# Service version (auto-detected if not set)
service_version = "1.0.0"

# Environment
environment = "production"

# Export interval
export_interval = "10s"

# Batch size
batch_size = 512
```

## Traces

roea-ai creates spans for:

### Agent Sessions

```
roea-agent
└── agent_session (span)
    ├── agent: claude-code
    ├── start_time: ...
    ├── process_count: 12
    └── child spans...
        ├── process_spawn
        ├── api_call
        └── file_operation
```

### API Calls

```
api_call (span)
├── endpoint: api.anthropic.com
├── method: POST
├── duration_ms: 450
├── bytes_sent: 1024
└── bytes_recv: 8192
```

### File Operations

```
file_operation (span)
├── operation: write
├── path: /src/main.rs
├── size: 2048
└── agent: claude-code
```

## Metrics

Exported metrics:

| Metric | Type | Description |
|--------|------|-------------|
| `roea_processes_total` | Counter | Total processes tracked |
| `roea_processes_active` | Gauge | Currently active processes |
| `roea_connections_total` | Counter | Total connections |
| `roea_connections_active` | Gauge | Active connections |
| `roea_file_ops_total` | Counter | Total file operations |
| `roea_agents_active` | Gauge | Active AI agents |
| `roea_event_latency_ms` | Histogram | Event processing latency |

## Backend Setup

### Jaeger

```yaml
# docker-compose.yml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "4317:4317"   # OTLP gRPC
      - "16686:16686" # UI
```

Configure roea-ai:
```toml
[telemetry.opentelemetry]
endpoint = "http://localhost:4317"
protocol = "grpc"
```

### Grafana Tempo

```yaml
# tempo.yaml
server:
  http_listen_port: 3200

distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"
```

### Honeycomb

```toml
[telemetry.opentelemetry]
endpoint = "https://api.honeycomb.io:443"
protocol = "grpc"
headers = { "x-honeycomb-team" = "your-api-key" }
```

### Datadog

```toml
[telemetry.opentelemetry]
endpoint = "http://localhost:4317"
protocol = "grpc"
# Datadog Agent must be running with OTLP receiver enabled
```

## Environment Variables

OpenTelemetry standard environment variables:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_SERVICE_NAME=roea-agent
export OTEL_RESOURCE_ATTRIBUTES="environment=production"
```

## Sampling

Configure trace sampling:

```toml
[telemetry.opentelemetry.sampling]
# Sampling strategy: always_on, always_off, ratio
strategy = "ratio"

# Sample ratio (0.0 to 1.0)
ratio = 0.1

# Always sample these events
always_sample = ["error", "agent_session"]
```

## Custom Attributes

Add custom attributes to all telemetry:

```toml
[telemetry.opentelemetry.attributes]
host_name = "dev-machine"
team = "platform"
custom_field = "value"
```

## Troubleshooting

### No Data in Backend

1. Check endpoint is reachable:
   ```bash
   curl -v http://localhost:4317
   ```

2. Check roea-agent logs:
   ```bash
   journalctl -u roea-agent | grep -i otlp
   ```

3. Verify configuration:
   ```bash
   roea-cli config show | grep opentelemetry
   ```

### High Latency

If export is slow:
- Increase `batch_size`
- Increase `export_interval`
- Check network latency to endpoint

### Missing Spans

If spans are missing:
- Check sampling configuration
- Verify span limits aren't exceeded
- Check for export errors in logs

## Performance Impact

OpenTelemetry export adds minimal overhead:

| Setting | Impact |
|---------|--------|
| Disabled | Baseline |
| Enabled (batch) | +1-2% CPU |
| Enabled (real-time) | +3-5% CPU |

Recommendations:
- Use batching (default)
- Sample in high-volume environments
- Use compression for remote endpoints

## See Also

- [Monitoring Guide](/docs/monitoring.md) - Internal monitoring
- [Configuration](/reference/configuration) - Full config reference
- [OpenTelemetry Docs](https://opentelemetry.io/docs/) - Official docs
