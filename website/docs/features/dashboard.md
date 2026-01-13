# Dashboard

The roea-ai dashboard provides an at-a-glance overview of AI agent activity on your system.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  roea-ai Dashboard                                    🟢 Connected │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │
│  │ 3 Agents    │  │ 47 Procs    │  │ 12 Conns    │               │
│  │   Active    │  │   Running   │  │   Active    │               │
│  └─────────────┘  └─────────────┘  └─────────────┘               │
│                                                                   │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │ Agent Activity (Last Hour)                                 │   │
│  │ ████████████████░░░░░░░░ Claude Code                      │   │
│  │ ██████████░░░░░░░░░░░░░░ Cursor                           │   │
│  │ ████░░░░░░░░░░░░░░░░░░░░ Aider                            │   │
│  └───────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Key Metrics

### Active Agents

Shows currently detected AI agents:

| Metric | Description |
|--------|-------------|
| **Count** | Number of active agent instances |
| **Types** | Which agents are running |
| **Duration** | How long each has been active |

### Process Count

Summary of process activity:

| Metric | Description |
|--------|-------------|
| **Running** | Currently active processes |
| **Exited** | Processes that ended recently |
| **Total** | Total processes tracked |
| **Agent processes** | Processes spawned by agents |

### Connection Count

Network connection summary:

| Metric | Description |
|--------|-------------|
| **Active** | Open connections |
| **Established** | Fully connected |
| **API calls** | Connections to known APIs |
| **Unknown** | Unclassified destinations |

## Activity Timeline

The timeline shows agent activity over time:

```
12:00 ────┬── Claude Code started
          ├── 3 processes spawned
          └── API connection opened

12:15 ────┬── Cursor activated
          ├── 8 helper processes
          └── Extension loaded

12:30 ────┬── Claude Code completed task
          ├── 5 files modified
          └── Session ended

12:45 ────┬── Aider started
          └── Python environment loaded
```

### Timeline Filters

- **Time range**: 1h, 4h, 24h, 7d
- **Agent filter**: Show specific agents
- **Event type**: Processes, connections, files

## Agent Cards

Each active agent has a detailed card:

```
┌─────────────────────────────────────┐
│ 🤖 Claude Code                      │
│ ─────────────────────────────────── │
│ Status: Running ● 45 min            │
│ Processes: 12 (8 active)            │
│ Network: 3 connections              │
│ Files: 127 operations               │
│                                     │
│ Recent Activity:                    │
│ • Modified src/main.rs              │
│ • Called Anthropic API              │
│ • Spawned git process               │
│                                     │
│ [View Details] [View Graph]         │
└─────────────────────────────────────┘
```

## Statistics Panel

### Today's Summary

```
┌─────────────────────────────────────┐
│ Today's Summary                     │
│ ─────────────────────────────────── │
│ Agent sessions:       8             │
│ Processes spawned:    234           │
│ API calls made:       156           │
│ Files modified:       89            │
│ Time active:          4h 32m        │
└─────────────────────────────────────┘
```

### Trends

Compare current activity to averages:

```
┌─────────────────────────────────────┐
│ vs Last 7 Days                      │
│ ─────────────────────────────────── │
│ Agent activity:    ▲ +23%           │
│ Process count:     ▼ -12%           │
│ API calls:         ▲ +45%           │
│ File operations:   ─ +2%            │
└─────────────────────────────────────┘
```

## Network Overview

Quick view of network activity:

```
┌─────────────────────────────────────┐
│ Network Connections                 │
│ ─────────────────────────────────── │
│ api.anthropic.com      12 calls     │
│ api.github.com          8 calls     │
│ registry.npmjs.org      3 calls     │
│ pypi.org                2 calls     │
│ ─────────────────────────────────── │
│ Total bandwidth: 2.4 MB             │
└─────────────────────────────────────┘
```

## File Activity Heat Map

See which directories are most active:

```
src/
├── components/  ████████████ (89 ops)
├── lib/         ██████ (42 ops)
├── utils/       ███ (18 ops)
└── types/       █ (7 ops)

tests/
└── unit/        ████ (23 ops)
```

## Alerts & Notifications

The dashboard shows important alerts:

```
⚠️ Alerts (2)
├── Unusual network destination detected
│   Process: node (PID 1234)
│   Destination: unknown-server.com
│
└── High file I/O activity
    Process: npm (PID 5678)
    Operations: 1,234 in last minute
```

### Alert Types

| Alert | Level | Description |
|-------|-------|-------------|
| Unknown network | Medium | Connection to unrecognized host |
| Sensitive file | High | Access to credential files |
| High activity | Low | Unusual process/file activity |
| Long session | Info | Agent running for extended time |

## Quick Actions

Perform common actions from the dashboard:

| Action | Description |
|--------|-------------|
| **Refresh** | Update all metrics |
| **Clear data** | Reset session data |
| **Export report** | Download activity report |
| **Settings** | Open preferences |

## Customization

### Widget Layout

Drag and drop widgets to customize:
- Resize widgets
- Show/hide specific widgets
- Save layout as preset

### Display Options

```toml
[ui.dashboard]
# Refresh interval
refresh_interval = "5s"

# Default time range
default_time_range = "1h"

# Show exited agents
show_exited_agents = true
exited_visible_duration = "1h"

# Widgets to display
widgets = [
  "active_agents",
  "process_summary",
  "network_overview",
  "activity_timeline",
  "file_heatmap"
]
```

### Metrics Thresholds

Configure when metrics turn yellow/red:

```toml
[ui.dashboard.thresholds]
# Process count
process_warning = 100
process_critical = 200

# Connection count
connection_warning = 50
connection_critical = 100

# File operations per minute
file_ops_warning = 500
file_ops_critical = 1000
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `R` | Refresh dashboard |
| `G` | Go to graph view |
| `T` | Toggle timeline |
| `1-9` | Select agent card |
| `?` | Show shortcuts |

## Export Dashboard

### Snapshot

Export dashboard state:

```bash
roea-cli dashboard snapshot --output dashboard.html
```

### Report

Generate activity report:

```bash
roea-cli dashboard report \
  --format pdf \
  --time-range 24h \
  --output report.pdf
```

### Scheduled Reports

Configure automated reports:

```toml
[reports]
enabled = true
schedule = "0 9 * * *"  # Daily at 9 AM
format = "pdf"
email = "team@example.com"
```

## Next Steps

- [Process Graph](/features/process-graph) - Interactive visualization
- [Search & Filtering](/features/search) - Find specific data
- [Configuration](/reference/configuration) - Customize roea-ai
