# Network Tracking

Network tracking monitors all network connections made by AI agents, helping you understand which APIs they call and identify unexpected network activity.

## Overview

roea-ai captures:

- **TCP connections**: HTTP/HTTPS API calls, WebSocket connections
- **UDP connections**: DNS queries, some telemetry
- **Unix sockets**: Local IPC communication
- **Connection states**: Connecting, established, closed

## What's Captured

For each connection, roea-ai records:

| Field | Description | Example |
|-------|-------------|---------|
| `pid` | Process making connection | 12345 |
| `protocol` | TCP, UDP, or Unix | TCP |
| `local_addr` | Local IP address | 192.168.1.100 |
| `local_port` | Local port | 54321 |
| `remote_addr` | Remote IP address | 104.18.7.47 |
| `remote_port` | Remote port | 443 |
| `state` | Connection state | Established |
| `endpoint_type` | Classification | LLM_API |

## Endpoint Classification

roea-ai automatically classifies connections:

### LLM API Endpoints

| Domain | Agent | Port |
|--------|-------|------|
| api.anthropic.com | Claude | 443 |
| api.openai.com | OpenAI/Cursor | 443 |
| api.cursor.sh | Cursor | 443 |
| api.continue.dev | Continue | 443 |

### Development Services

| Domain | Service | Port |
|--------|---------|------|
| api.github.com | GitHub API | 443 |
| github.com | GitHub | 443 |
| gitlab.com | GitLab | 443 |
| bitbucket.org | Bitbucket | 443 |

### Package Registries

| Domain | Registry | Port |
|--------|----------|------|
| registry.npmjs.org | npm | 443 |
| pypi.org | PyPI | 443 |
| crates.io | Crates.io | 443 |
| rubygems.org | RubyGems | 443 |

### Telemetry

| Domain | Service | Port |
|--------|---------|------|
| sentry.io | Sentry | 443 |
| segment.com | Segment | 443 |
| amplitude.com | Amplitude | 443 |

## Platform-Specific Implementation

### Linux

Parses `/proc/net/tcp`, `/proc/net/tcp6`, `/proc/net/udp`, `/proc/net/unix`:

```bash
# What roea-ai reads
cat /proc/net/tcp
#   sl  local_address rem_address   st tx_queue rx_queue ...
#    0: 0100007F:C350 00000000:0000 0A 00000000:00000000 ...
```

Advantages:
- Complete connection visibility
- No elevated privileges needed
- Real-time updates

### macOS

Uses `lsof` and system APIs:

```bash
# Similar to
lsof -i -n -P | grep <pid>
```

### Windows

Uses `netstat` equivalent APIs:

```powershell
# Similar to
Get-NetTCPConnection -OwningProcess <pid>
```

## Connection Lifecycle

roea-ai tracks connection state transitions:

```
[New] → [Connecting] → [Established] → [Closed]
```

### State Definitions

| State | Meaning |
|-------|---------|
| Connecting | TCP SYN sent, waiting for response |
| Established | Connection active |
| CloseWait | Remote closed, local not yet closed |
| TimeWait | Connection closing, waiting for timeout |
| Closed | Connection fully terminated |

## Use Cases

### Audit API Calls

See which AI APIs your agents use:

```
Claude Code → api.anthropic.com:443 (Established)
Claude Code → api.github.com:443 (Established)
```

### Detect Unexpected Traffic

Identify connections to unknown endpoints:

```
⚠️ Unknown endpoint: 185.199.108.133:443
   Process: cursor-helper (PID: 4567)
```

### Monitor Data Egress

Track what data might be leaving:

```
Total connections this session:
- api.anthropic.com: 156 requests
- api.github.com: 23 requests
- registry.npmjs.org: 89 requests
```

## Filtering

### By Endpoint Type

```
type:llm_api     // Only AI API calls
type:github      // GitHub traffic
type:registry    // Package managers
type:telemetry   // Analytics/telemetry
type:unknown     // Unclassified
```

### By Protocol

```
protocol:tcp
protocol:udp
protocol:unix
```

### By State

```
state:established
state:closed
```

### By Process

```
pid:12345
process:node
```

## Visualization

### Network Overlay

Enable network overlay on the process graph:

1. Click "Show Networks" toggle
2. Connections appear as curved lines from processes
3. Line color indicates endpoint type:
   - Blue: LLM APIs
   - Green: GitHub
   - Orange: Registries
   - Red: Unknown

### Connection List

View detailed connection list in the details panel:

```
┌──────────────────────────────────────────────────────────┐
│ Network Connections (5)                                   │
├──────────────────────────────────────────────────────────┤
│ api.anthropic.com:443     TCP  Established  LLM_API      │
│ api.github.com:443        TCP  Established  GitHub       │
│ registry.npmjs.org:443    TCP  Closed       Registry     │
│ 127.0.0.1:50051           TCP  Established  Localhost    │
│ /tmp/socket.sock          Unix Established  IPC          │
└──────────────────────────────────────────────────────────┘
```

## Privacy Considerations

roea-ai does NOT capture:

- Request/response bodies
- HTTP headers
- Authentication tokens
- TLS/SSL content

Only connection metadata (IPs, ports, states) is recorded.

## Performance

Network monitoring overhead:

| Platform | Method | Overhead |
|----------|--------|----------|
| Linux | /proc parsing | < 0.1% CPU |
| macOS | lsof | < 1% CPU |
| Windows | netstat API | < 1% CPU |

Polling interval: 500ms (configurable)

## API Reference

### Get Connections

```rust
// Get all connections for a process
let connections = monitor.get_connections_for_pid(12345).await?;

for conn in connections {
    println!("{} → {}:{}",
        conn.local_addr,
        conn.remote_addr,
        conn.remote_port
    );
}
```

### Watch Connection Events

```rust
// Stream connection events
let mut stream = monitor.watch_connections().await?;
while let Some(event) = stream.next().await {
    match event {
        ConnectionEvent::New(c) => println!("Connect: {}", c.remote_addr),
        ConnectionEvent::Closed(c) => println!("Closed: {}", c.remote_addr),
    }
}
```

## Troubleshooting

### Connections Not Appearing

1. **Check process has connections**: `lsof -i -P -n | grep <pid>`
2. **Verify poll timing**: Connection may have closed quickly
3. **Check permissions**: Some platforms need elevated access

### Wrong Endpoint Classification

1. **Add custom classification**: Edit endpoint rules
2. **Report issue**: Help improve default classifications

### Missing Unix Sockets

1. **Check platform**: Unix sockets fully supported on Linux
2. **Verify path access**: May need read access to socket paths

## Related

- [Process Monitoring](/features/process-monitoring) - Track spawned processes
- [File Access](/features/file-access) - Monitor file operations
- [Configuration](/reference/configuration) - Configure network monitoring
