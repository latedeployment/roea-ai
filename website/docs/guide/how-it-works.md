# How It Works

roea-ai is an EDR-like (Endpoint Detection and Response) observability tool that monitors AI coding agents on your system. This page explains the technical architecture and how roea-ai captures and presents telemetry data.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Desktop UI (Tauri)                   │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────────┐ │
│  │  Sidebar    │  │ Process     │  │   Details Panel      │ │
│  │  (Agents)   │  │ Graph (D3)  │  │   (Connections/Files)│ │
│  └─────────────┘  └─────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │ gRPC
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    roea-agent (Rust Daemon)                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │
│  │ Process  │  │ Network  │  │   File   │  │  Signature  │  │
│  │ Monitor  │  │ Monitor  │  │ Monitor  │  │  Matcher    │  │
│  └──────────┘  └──────────┘  └──────────┘  └─────────────┘  │
│                              │                               │
│                       ┌──────┴──────┐                       │
│                       │   DuckDB    │                       │
│                       │   Storage   │                       │
│                       └─────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. roea-agent (Monitoring Daemon)

The `roea-agent` is a Rust-based daemon that runs in the background and collects telemetry from your system. It consists of several monitors:

#### Process Monitor

The process monitor tracks all processes on your system, building a real-time process tree. It uses different strategies depending on your platform:

| Platform | Primary Method | Fallback |
|----------|---------------|----------|
| **Linux** | eBPF tracepoints | sysinfo polling |
| **macOS** | Endpoint Security API | sysinfo polling |
| **Windows** | ETW (Event Tracing) | sysinfo polling |

The eBPF backend on Linux provides:
- Sub-millisecond detection latency
- Kernel-level event capture
- Minimal CPU overhead (<1%)

#### Network Monitor

Tracks all network connections made by processes:
- TCP and UDP connections
- Unix domain sockets
- DNS queries (where possible)
- Connection state transitions

Data sources:
- Linux: `/proc/net/tcp`, `/proc/net/udp`, eBPF socket tracepoints
- macOS: Network Extension framework
- Windows: ETW network events

#### File Monitor

Tracks file system operations:
- File opens, reads, writes
- File creation and deletion
- Directory operations

Includes noise filtering to ignore:
- System files (`/proc`, `/sys`, etc.)
- Temporary files
- Build artifacts (`.o`, `.pyc`, etc.)

### 2. Signature Matcher

The signature matcher identifies AI coding agents from the process stream. It uses a rule-based system defined in YAML configuration files:

```yaml
name: claude-code
display_name: Claude Code
process_patterns:
  - "^claude$"
  - "claude-code"
command_patterns:
  - "--api-key"
  - "chat"
  - "code"
network_endpoints:
  - "api.anthropic.com"
track_children: true
```

See [Agent Detection](/guide/agent-detection) for details on how agents are identified.

### 3. DuckDB Storage

All telemetry is stored locally in an embedded DuckDB database. DuckDB provides:
- Fast analytical queries
- Columnar storage for efficient compression
- SQL interface for complex queries
- Time-series optimized storage

Database schema:
- `processes` - Process lifecycle events
- `connections` - Network connection events
- `file_ops` - File system operations
- `agents` - Detected AI agents

### 4. Desktop UI

The desktop UI is built with Tauri (Rust + React) and provides:
- Real-time process tree visualization (D3.js)
- Network connection graph overlay
- Search and filtering capabilities
- Export to JSON/CSV

Communication with the daemon is via gRPC for efficient streaming.

## Data Flow

```
1. OS Kernel Events
       │
       ▼
2. Platform Monitors (eBPF/ETW/ES)
       │
       ▼
3. Event Processing & Deduplication
       │
       ▼
4. Signature Matching (Agent Detection)
       │
       ▼
5. DuckDB Storage (Local)
       │
       ▼
6. gRPC Streaming to UI
       │
       ▼
7. React Components (D3.js Visualization)
```

## Performance Characteristics

roea-ai is designed for minimal overhead:

| Metric | Target | Typical |
|--------|--------|---------|
| CPU (idle) | <0.5% | 0.1-0.3% |
| CPU (active monitoring) | <2% | 0.5-1.5% |
| Memory footprint | <100MB | 50-80MB |
| Event latency (p95) | <50ms | 10-30ms |
| Storage growth | ~10MB/hour | Varies |

## Security Model

roea-ai operates with these security principles:

1. **Local-only storage** - All data stays on your machine
2. **No network transmission** - No telemetry sent to external servers
3. **Minimal permissions** - Only requests necessary OS permissions
4. **Open source** - Fully auditable code

Required permissions:
- **Linux**: CAP_BPF or root for eBPF, read access to `/proc`
- **macOS**: Full Disk Access for file monitoring
- **Windows**: Administrator for ETW subscriptions

## Integration Points

roea-ai can integrate with external systems:

### OpenTelemetry Export

Export telemetry in OTLP format to:
- Jaeger
- Zipkin
- Any OTLP-compatible backend

### osquery

Use osquery's SQL interface for advanced queries:
```sql
SELECT * FROM processes WHERE name LIKE '%claude%';
```

See [osquery Integration](/reference/osquery) for setup instructions.

## Next Steps

- [Agent Detection](/guide/agent-detection) - How AI agents are identified
- [Data Storage](/guide/storage) - Storage configuration and retention
- [System Requirements](/guide/requirements) - Platform requirements
