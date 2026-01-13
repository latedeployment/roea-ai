# Monitoring & Observability Guide

This document describes the monitoring and observability infrastructure for roea-ai.

## Overview

roea-ai includes built-in observability features:
- **Error Tracking**: Sentry integration for production error monitoring
- **Internal Metrics**: Custom metrics collection for performance monitoring
- **CI/CD Metrics**: GitHub Actions workflow tracking
- **Build Time Monitoring**: Automatic build duration tracking
- **Alerting**: Failure notifications and flakiness detection

## Error Tracking with Sentry

### Setup

1. Create a Sentry project at https://sentry.io
2. Get your DSN from Project Settings > Client Keys
3. Set the environment variable:
   ```bash
   export SENTRY_DSN="https://key@sentry.io/project-id"
   ```

### Configuration

```rust
use roea_agent::{init_sentry, SentryConfig};

// Default configuration (reads from SENTRY_DSN env var)
let _sentry = init_sentry(SentryConfig::default());

// Or with custom configuration
let _sentry = init_sentry(
    SentryConfig::new("https://key@sentry.io/123")
        .environment("production")
        .release("1.0.0")
        .sample_rate(1.0)
        .traces_sample_rate(0.1)
        .tag("deployment", "self-hosted")
);
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SENTRY_DSN` | Sentry DSN URL | None (disabled) |
| `SENTRY_ENVIRONMENT` | Environment name | "development" |

### Usage

```rust
use roea_agent::observability::{
    capture_error,
    capture_message,
    add_breadcrumb,
    start_transaction,
};

// Capture errors
if let Err(e) = risky_operation() {
    capture_error(&e, Some("During startup"));
}

// Add breadcrumbs for debugging context
add_breadcrumb("process", "Started monitoring", sentry::Level::Info);

// Capture important events
capture_message("Configuration loaded", sentry::Level::Info);

// Performance transactions
if let Some(tx) = start_transaction("process_scan", "monitor") {
    // Perform monitored operation
    // Transaction ends when tx is dropped
}
```

### Privacy

- **No PII**: Sentry is configured to not send personally identifiable information
- **Local data**: Process names and paths are sent for debugging but no user data
- **Opt-out**: Don't set `SENTRY_DSN` to disable error tracking entirely

## Internal Metrics

roea-ai collects internal performance metrics using a lightweight metrics system.

### Available Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `roea_processes_tracked_total` | Counter | Total processes tracked |
| `roea_connections_tracked_total` | Counter | Total connections tracked |
| `roea_file_ops_tracked_total` | Counter | Total file operations |
| `roea_agents_detected_total` | Counter | AI agents detected |
| `roea_active_processes` | Gauge | Currently active processes |
| `roea_active_connections` | Gauge | Currently active connections |
| `roea_process_monitor_latency_ms` | Histogram | Process monitor cycle time |
| `roea_network_monitor_latency_ms` | Histogram | Network monitor cycle time |
| `roea_grpc_request_latency_ms` | Histogram | gRPC request latency |
| `roea_errors_total` | Counter | Total errors encountered |

### Usage

```rust
use roea_agent::observability::{
    processes_tracked,
    active_processes,
    process_monitor_latency,
    metrics,
};

// Increment counters
processes_tracked().inc();

// Update gauges
active_processes().set(42);
active_processes().inc();
active_processes().dec();

// Record latencies
process_monitor_latency().observe(12.5); // milliseconds

// Get a snapshot of all metrics
let snapshot = metrics().snapshot();
println!("Metrics: {:?}", snapshot);
```

### Exporting Metrics

Get a JSON snapshot of all metrics:

```rust
let snapshot = metrics().snapshot();
let json = serde_json::to_string_pretty(&snapshot)?;
```

## CI/CD Metrics

GitHub Actions workflows automatically collect and report metrics.

### Workflow: `.github/workflows/metrics.yml`

This workflow:
- Triggers on completion of CI, Nightly, Release, and Security workflows
- Calculates and reports build duration
- Generates weekly metrics summaries
- Alerts on repeated failures
- Detects test flakiness

### Metrics Collected

| Metric | Description |
|--------|-------------|
| Build Duration | Time from start to completion |
| Success Rate | Percentage of successful builds |
| Failure Count | Number of failed builds |
| Flakiness Rate | Ratio of failures to total runs |

### Weekly Report

Every Monday at 8 AM UTC, a metrics report is generated with:
- CI workflow success rate
- Nightly build success rate
- Security scan status
- Overall health summary

### Alerts

Alerts are generated for:
- Individual workflow failures
- 3+ consecutive failures on the same branch
- Build times exceeding 30 minutes
- Flakiness rate above 20%

## Setting Up Monitoring

### Development

For local development, minimal monitoring is needed:

```bash
# Enable debug logging
export RUST_LOG=debug

# Run without Sentry
cargo run
```

### Staging

```bash
export SENTRY_DSN="https://key@sentry.io/staging-project"
export SENTRY_ENVIRONMENT="staging"
```

### Production

```bash
export SENTRY_DSN="https://key@sentry.io/production-project"
export SENTRY_ENVIRONMENT="production"
```

## GitHub Repository Secrets

For CI/CD monitoring, configure these secrets (optional):

| Secret | Description |
|--------|-------------|
| `SENTRY_DSN` | Sentry DSN for CI error tracking |
| `SLACK_WEBHOOK_URL` | Slack webhook for notifications |
| `DISCORD_WEBHOOK_URL` | Discord webhook for notifications |

## Dashboards

### Recommended Grafana Cloud Setup

1. Sign up for free tier at https://grafana.com/products/cloud/
2. Create a dashboard with:
   - Build success rate over time
   - Build duration trends
   - Error rate from Sentry
   - Active issues count

### Sentry Dashboard

Configure your Sentry project with:
- Alert rules for new errors
- Performance monitoring enabled
- Release tracking from git tags

## Troubleshooting

### Sentry Not Receiving Events

1. Check DSN is correct: `echo $SENTRY_DSN`
2. Check network connectivity to sentry.io
3. Enable debug mode: `SentryConfig::default().debug(true)`
4. Check sample rate is not 0

### Missing CI Metrics

1. Ensure metrics workflow is enabled
2. Check workflow permissions (needs `actions: read`)
3. Verify workflow triggers are correct

### High Memory Usage in Metrics

The metrics system uses atomic operations and is designed to be lightweight.
If memory usage is high:
1. Check for metric cardinality issues (too many unique labels)
2. Consider reducing histogram bucket count
3. Periodically reset metrics if needed
