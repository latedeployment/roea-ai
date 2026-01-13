# Process Monitoring

Process monitoring is the core feature of roea-ai. It tracks all processes spawned by AI coding agents, their relationships, and lifecycle events.

## How It Works

roea-ai monitors processes using platform-specific backends:

| Platform | Method | Latency | Completeness |
|----------|--------|---------|--------------|
| Linux (eBPF) | Kernel tracepoints | < 1ms | Complete |
| Linux (fallback) | sysinfo polling | 100ms | Complete |
| macOS | sysinfo polling | 100ms | Complete |
| Windows | sysinfo polling | 100ms | Complete |

### Linux eBPF Monitoring

On Linux with eBPF support, roea-ai hooks directly into the kernel:

```c
// Tracepoints monitored
sched_process_exec   // New process execution
sched_process_exit   // Process termination
```

This provides:
- Zero-latency detection of new processes
- Complete capture of short-lived processes
- Minimal CPU overhead (< 0.1%)

### Polling-Based Monitoring

On other platforms, roea-ai polls process information:

1. Query active processes via OS APIs
2. Diff against previous snapshot
3. Detect new/exited processes
4. Build process tree relationships

Polling interval: 100ms (configurable)

## Process Information Captured

For each process, roea-ai captures:

| Field | Description | Example |
|-------|-------------|---------|
| `pid` | Process ID | 12345 |
| `ppid` | Parent process ID | 1234 |
| `name` | Process name | "node" |
| `exe_path` | Full executable path | /usr/bin/node |
| `cmdline` | Command line arguments | node index.js --port 3000 |
| `cwd` | Current working directory | /home/user/project |
| `start_time` | Process start timestamp | 2024-01-15T10:30:00Z |
| `exit_time` | Process exit timestamp | 2024-01-15T10:35:00Z |
| `exit_code` | Exit status code | 0 |
| `uid` | User ID | 1000 |
| `agent_type` | Detected AI agent type | "claude-code" |

## Process Tree Construction

roea-ai builds a hierarchical process tree:

```
Claude Code (PID: 1000)
├── bash (PID: 1001)
│   ├── grep (PID: 1002)
│   └── cat (PID: 1003)
├── node (PID: 1004)
│   └── npm (PID: 1005)
└── git (PID: 1006)
```

### Tree Building Algorithm

1. **Root Detection**: Find processes matching agent signatures
2. **Child Discovery**: Walk process parent chain to find children
3. **Tree Update**: Add/remove nodes as processes spawn/exit
4. **Orphan Handling**: Re-parent orphaned children to nearest ancestor

### PID Reuse Handling

Linux reuses PIDs aggressively. roea-ai handles this by:

- Assigning unique UUIDs to each process incarnation
- Tracking process start times
- Detecting PID reuse via start time comparison

## Visualization

The process graph provides multiple layout options:

### Force-Directed Layout

Default layout using D3.js physics simulation:

- Natural clustering of related processes
- Draggable nodes
- Zoom and pan support
- Real-time updates without layout reset

### Tree Layout

Hierarchical top-down view:

```
         [Claude]
        /    |    \
    [bash] [node] [git]
      |       |
   [grep]  [npm]
```

Best for understanding parent-child relationships.

### Radial Layout

Circular arrangement:

- Root at center
- Children in concentric rings
- Good for deep process trees

## Filtering and Search

### By Agent Type

Filter to show only specific agents:

```typescript
// UI filter
agent:claude-code
agent:cursor
```

### By Process Name

Search by process name:

```
name:node
name:npm
```

### By Status

Filter active or exited processes:

```
status:active
status:exited
```

### By Time Range

Show processes from specific time:

```
since:1h
since:30m
```

## Performance Considerations

### High Process Churn

AI agents often spawn many short-lived processes:

| Scenario | Processes/sec | roea-ai Handling |
|----------|---------------|------------------|
| Normal coding | 1-10 | No issues |
| npm install | 50-100 | Buffered updates |
| Large build | 100+ | Batched rendering |

roea-ai batches UI updates to maintain 60fps even under heavy load.

### Memory Usage

Process data is stored efficiently:

- Active processes: In-memory tree structure
- Historical data: DuckDB with automatic cleanup
- Default retention: 7 days

## API Reference

### Get Process Tree

```rust
// Rust API
let tree = monitor.get_process_tree().await?;
for process in tree.iter() {
    println!("{}: {}", process.pid, process.name);
}
```

### Watch Process Events

```rust
// Stream real-time events
let mut stream = monitor.watch_processes().await?;
while let Some(event) = stream.next().await {
    match event {
        ProcessEvent::Spawn(p) => println!("New: {}", p.name),
        ProcessEvent::Exit(p) => println!("Exit: {}", p.name),
    }
}
```

## Troubleshooting

### Processes Not Detected

1. **Check agent is running**: `ps aux | grep <agent-name>`
2. **Verify signature**: Agent may need custom signature
3. **Check permissions**: May need elevated privileges

### Tree Missing Children

1. **Polling delay**: Short-lived processes may be missed
2. **Enable eBPF**: Use eBPF for complete capture
3. **Increase poll rate**: Lower polling interval

### High CPU Usage

1. **Check process count**: Many processes increase load
2. **Reduce poll rate**: Increase polling interval
3. **Enable eBPF**: Much lower overhead

## Related

- [Agent Detection](/guide/agent-detection) - How agents are identified
- [Network Tracking](/features/network-tracking) - Monitor network connections
- [Linux eBPF Setup](/reference/ebpf) - Enable kernel-level monitoring
