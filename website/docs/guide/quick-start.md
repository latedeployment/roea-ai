# Quick Start

This guide will help you get roea-ai running and monitoring AI agents in under 5 minutes.

## Prerequisites

- roea-ai installed ([Installation Guide](/guide/installation))
- At least one AI coding agent installed (Claude Code, Cursor, etc.)

## Step 1: Start the Agent Daemon

The roea-agent daemon runs in the background and collects monitoring data.

```bash
# Start the daemon
roea-agent

# You should see:
# INFO roea_agent: Starting roea-agent daemon...
# INFO roea_agent::grpc: gRPC server listening on [::1]:50051
# INFO roea_agent::monitor: Process monitoring started
```

::: tip Running as a Service
For production use, consider running roea-agent as a system service. See [Daemon Setup](/reference/daemon) for details.
:::

## Step 2: Launch the UI

Open the roea-ai desktop application:

```bash
# On macOS/Linux
roea-ui

# Or on macOS, open from Applications
open /Applications/roea-ai.app
```

The UI will automatically connect to the daemon and display:

- **Connection status**: Green indicator when connected
- **Empty process list**: No agents detected yet

## Step 3: Start an AI Agent

Now use any supported AI coding agent. For this example, we'll use Claude Code:

```bash
# In a new terminal, start Claude Code
claude

# Or use Cursor
cursor .

# Or use Aider
aider
```

## Step 4: Watch the Visualization

Return to the roea-ai UI. You should now see:

1. **Sidebar**: Shows detected agents (e.g., "Claude Code")
2. **Process Graph**: Visual tree of processes spawned
3. **Stats Bar**: Counts of processes, connections, and file operations

### Understanding the Process Graph

The graph shows processes as nodes connected by lines:

- **Blue nodes**: AI agent processes
- **Gray nodes**: Child processes
- **Dashed outlines**: Exited processes
- **Lines**: Parent → Child relationships

Click any node to see details in the right panel.

## Step 5: Explore the Data

### View Network Connections

Click on any process node to see its network connections:

```
api.anthropic.com:443  (TCP, Established)
api.github.com:443     (TCP, Established)
localhost:50051        (TCP, Established)
```

### View File Operations

The file tab shows which files the agent accessed:

```
/home/user/project/src/main.rs     (Read)
/home/user/project/package.json    (Read)
/home/user/project/Cargo.toml      (Write)
```

### Search and Filter

Use the search bar to filter processes:

- Search by name: `node`
- Search by PID: `12345`
- Filter by agent: Click agent names in sidebar

## Example Session

Here's what a typical Claude Code session looks like:

```
Claude Code (PID: 1234)
├── node (PID: 1235)
│   ├── npm (PID: 1236)
│   │   └── node (PID: 1237)
│   └── git (PID: 1238)
├── bash (PID: 1239)
│   ├── grep (PID: 1240)
│   └── rg (PID: 1241)
└── cargo (PID: 1242)
    └── rustc (PID: 1243)
```

## Common Tasks

### Export Session Data

Export the current view as JSON or CSV:

1. Click the **Export** button in the toolbar
2. Choose format (JSON/CSV)
3. Select destination file

### Switch Graph Layouts

Change the visualization style:

- **Force Layout**: Default, physics-based positioning
- **Tree Layout**: Hierarchical top-down view
- **Radial Layout**: Circular arrangement

Use the layout dropdown in the toolbar.

### Filter by Agent

Monitor specific agents:

1. Click an agent name in the sidebar
2. The graph filters to show only that agent's processes
3. Click "All" to show all agents again

## Troubleshooting

### No Agents Detected

If roea-ai doesn't detect your AI agent:

1. Ensure the agent is actually running (check `ps aux | grep <agent-name>`)
2. Check that the agent signature is defined
3. See [Agent Detection](/guide/agent-detection) for custom signatures

### UI Won't Connect

If the UI shows "Disconnected":

1. Verify the daemon is running: `ps aux | grep roea-agent`
2. Check the daemon port: Should be 50051 by default
3. Check for firewall rules blocking localhost connections

### Missing File Operations

File monitoring may be limited on some platforms:

- **macOS**: Requires Full Disk Access permission
- **Linux**: More complete with eBPF (see [eBPF Setup](/reference/ebpf))
- **Windows**: Full support coming soon

## Next Steps

Now that you have roea-ai running:

- [Learn about process monitoring](/features/process-monitoring)
- [Set up network tracking](/features/network-tracking)
- [Configure agent signatures](/reference/configuration)
- [Export data to OpenTelemetry](/reference/opentelemetry)
