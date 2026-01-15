# Security Dashboard UI Redesign

## Current Problems

1. **Force-directed graph is confusing** - Nodes float around, hard to understand relationships
2. **No event stream** - Users can't see real-time activity like file operations or network connections
3. **No search for files/IPs** - Critical for security investigation workflows
4. **Not EDR-like** - Doesn't match user expectations for security tooling

## Proposed Architecture

```
┌─────────────────────────────────────┐
│         Main Layout                 │
├─────────────────────────────────────┤
│     Header with Status              │
├──────────────┬──────────────────────┤
│              │                      │
│  Process     │  Event Table +       │
│  Tree Panel  │  Details Panel       │
│              │                      │
├──────────────┴──────────────────────┤
│        Quick Stats Bar              │
└─────────────────────────────────────┘
```

## Core Components

### 1. Process Tree (Left Panel) - Replace Force Graph

A collapsible tree view showing parent-child process relationships:

```
[cursor] cursor-server (PID 221961)
  ├─ [node] server-main.js (PID 221973)
  │    ├─ [node] extensionHost (PID 222597)
  │    └─ [node] fileWatcher (PID 222048)
  └─ [cursor] command-shell (PID 274645)

[claude] claude --dangerously-skip-permissions (PID 272848)
  ├─ [bun] Bun Pool 1 (PID 298065)
  └─ [bash] shell session (PID 74202)
```

Features:
- Collapsible nodes (expand/collapse children)
- Color-coded by agent type
- Status indicators (green dot = running, gray = exited)
- Click to select and show details
- Right-click context menu for actions

### 2. Event Table (Center/Right) - New Component

Real-time event stream like EDR/SIEM tools:

| Time | Process | Event | Details |
|------|---------|-------|---------|
| 14:51:23 | claude (272848) | FILE_WRITE | /home/user/project/src/main.rs |
| 14:51:22 | claude (272848) | NETWORK | api.anthropic.com:443 ESTABLISHED |
| 14:51:20 | cursor (221973) | SPAWN | node extensionHost |
| 14:51:18 | claude (272848) | FILE_READ | /home/user/.bashrc |

Features:
- Live auto-scroll with pause on hover
- Event type icons/badges (file, network, process)
- Severity coloring (info=gray, warning=yellow, suspicious=red)
- Click row to expand details
- Infinite scroll with virtualization for performance

### 3. Advanced Search Bar - Enhanced

```
┌─────────────────────────────────────────────────────────────────┐
│ 🔍 Search files, IPs, processes...                    [Filters] │
├─────────────────────────────────────────────────────────────────┤
│ Type: [x] Files  [x] Network  [x] Processes                     │
│ Agent: [x] Claude  [x] Cursor  [ ] All                          │
│ Time:  [Last 1h ▼]  IP: [___________]  Path: [___________]     │
└─────────────────────────────────────────────────────────────────┘
```

Search capabilities:
- **File search**: `path:/home/user/*.rs` or `file:main.rs`
- **IP search**: `ip:142.250.` or `ip:api.anthropic.com`
- **Process search**: `name:node` or `pid:12345`
- **Combined**: `agent:claude file:*.py`

### 4. Details Panel (Right Sidebar) - Enhanced

When a process or event is selected:

```
┌─ Process Details ──────────────────┐
│ claude (PID 272848)           [x] │
│ Status: ● Running                  │
│ Agent Type: Claude Code            │
├────────────────────────────────────┤
│ PROCESS INFO                       │
│ Started: 2026-01-15 14:44:23       │
│ User: omer                         │
│ CWD: /home/omer/src/roea-ai        │
│ Command: claude --dangerously...   │
├────────────────────────────────────┤
│ NETWORK (12 connections)           │
│ • api.anthropic.com:443 ESTABLISHED│
│ • github.com:443 ESTABLISHED       │
├────────────────────────────────────┤
│ FILES (47 operations)              │
│ • /src/main.rs (WRITE)             │
│ • /Cargo.toml (READ)               │
└────────────────────────────────────┘
```

## File Changes

### New Components to Create

1. **`crates/roea-ui/src/components/ProcessTree.tsx`** - Hierarchical tree view replacing ProcessGraph
2. **`crates/roea-ui/src/components/EventTable.tsx`** - Real-time event stream table
3. **`crates/roea-ui/src/components/EventRow.tsx`** - Individual event row with expand/collapse

### Components to Modify

1. **`crates/roea-ui/src/App.tsx`** - New layout with split view
2. **`crates/roea-ui/src/components/SearchBar.tsx`** - Add file path and IP search capabilities
3. **`crates/roea-ui/src/components/DetailsPanel.tsx`** - Add network/files tabs
4. **`crates/roea-ui/src/lib/types.ts`** - Add Event type definitions

### Components to Remove

1. **`crates/roea-ui/src/components/ProcessGraph.tsx`** - Replace with ProcessTree (remove D3 dependency)

### Backend Changes (Minor)

1. **`crates/roea-agent/src/grpc/mod.rs`** - Ensure `query_file_ops` returns proper data
2. **`crates/roea-ui/src-tauri/src/grpc_client.rs`** - Add `get_file_ops` method if missing

## UI Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│ [Logo] roea-ai          ● Connected    Uptime: 2h 15m        [⚙️]   │
├──────────────────────────────────────────────────────────────────────┤
│ 🔍 Search files, IPs, processes...                         [Filters]│
├────────────────────┬─────────────────────────────────────────────────┤
│                    │                                                 │
│  PROCESS TREE      │  EVENTS                              [Details] │
│  ─────────────     │  ─────────────────────────────────   ───────── │
│  ▼ cursor          │  14:51:23 FILE  claude  /src/main.rs │ Process │
│    ├─ node         │  14:51:22 NET   claude  anthropic    │ claude  │
│    └─ node         │  14:51:20 PROC  cursor  spawn node   │         │
│  ▼ claude          │  14:51:18 FILE  claude  .bashrc      │ Network │
│    ├─ bun          │  14:51:15 NET   cursor  github.com   │ 12 conn │
│    └─ bash         │  ...                                  │         │
│                    │                                       │ Files   │
│                    │                                       │ 47 ops  │
├────────────────────┴─────────────────────────────────────────────────┤
│ Processes: 8 │ Agents: 2 │ Events: 1,247 │ Files: 47 │ Connections: 12│
└──────────────────────────────────────────────────────────────────────┘
```

## Design System

Keep existing dark theme but add:
- Event type badges with distinct colors:
  - FILE operations: blue
  - NETWORK events: purple  
  - PROCESS events: green
  - SUSPICIOUS: red/orange
- Tree expand/collapse icons (▶ / ▼)
- Better status indicators
- Monospace font for paths/IPs/PIDs (`font-family: monospace`)

## Implementation Order

1. Create `ProcessTree.tsx` component with basic hierarchy
2. Create `EventTable.tsx` with mock data
3. Update `App.tsx` layout to split view
4. Enhance `SearchBar.tsx` with file/IP filters
5. Wire up real data from backend
6. Enhance `DetailsPanel.tsx` with tabs
7. Remove `ProcessGraph.tsx` and D3 dependency
8. Polish and test
