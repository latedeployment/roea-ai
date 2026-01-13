# Process Tree Graph

The process tree graph is the centerpiece of roea-ai's visualization. It shows your AI agents and their spawned processes as an interactive, real-time graph.

## Overview

```
                    ┌─────────┐
                    │ Terminal│
                    └────┬────┘
                         │
                    ┌────▼────┐
                    │ Claude  │  ← AI Agent (highlighted)
                    │  Code   │
                    └────┬────┘
              ┌──────────┼──────────┐
              │          │          │
         ┌────▼────┐┌────▼────┐┌────▼────┐
         │  bash   ││  node   ││  git    │
         └────┬────┘└─────────┘└─────────┘
              │
         ┌────▼────┐
         │  npm    │
         └─────────┘
```

## Graph Elements

### Nodes

Each node represents a process:

| Element | Meaning |
|---------|---------|
| **Circle size** | Process importance/activity |
| **Border color** | Agent type (Claude=blue, Cursor=purple, etc.) |
| **Fill color** | Process status (running=green, exited=gray) |
| **Icon** | Agent icon for detected agents |
| **Label** | Process name and PID |

### Edges

Lines between nodes show relationships:

| Line Type | Meaning |
|-----------|---------|
| **Solid line** | Parent-child relationship |
| **Dashed line** | Network connection (overlay) |
| **Arrow direction** | Parent → Child |
| **Line thickness** | Number of connections |

## Layout Options

### Force-Directed (Default)

Physics-based layout that naturally separates clusters:

- Agents repel each other
- Children stay near parents
- Good for general exploration

```
Settings:
├── Link distance: [100]
├── Charge strength: [-300]
├── Center force: [0.1]
└── Collision radius: [30]
```

### Tree Layout

Hierarchical layout showing parent-child relationships clearly:

- Root processes at top
- Children below parents
- Good for understanding process trees

```
Settings:
├── Orientation: [Top to Bottom ▼]
├── Node spacing: [50]
└── Level spacing: [100]
```

### Radial Layout

Circular layout with root processes at center:

- Agents at center
- Children in outer rings
- Good for large process trees

```
Settings:
├── Ring spacing: [80]
└── Angle spread: [360°]
```

## Interactions

### Node Selection

Click a node to:
- Highlight the node and its connections
- Show process details in the Details Panel
- Display network connections
- Show file access history

### Pan and Zoom

Navigate the graph:

| Action | Mouse | Keyboard |
|--------|-------|----------|
| Pan | Click + drag | Arrow keys |
| Zoom in | Scroll up | `+` or `=` |
| Zoom out | Scroll down | `-` |
| Reset view | Double-click background | `0` |
| Fit all | - | `F` |

### Node Dragging

Drag nodes to rearrange:
- Click and hold a node
- Drag to new position
- Node stays pinned until reset

### Context Menu

Right-click a node for options:
- **Copy PID** - Copy process ID
- **Copy command** - Copy full command line
- **View details** - Open details panel
- **Kill process** - Terminate process (requires permission)
- **Hide** - Hide from graph
- **Focus subtree** - Show only this process and children

## Network Overlay

Toggle the network overlay to see connections:

```
┌───────────┐                    ┌───────────┐
│  Claude   │───── HTTPS ────────│ Anthropic │
│   Code    │                    │    API    │
└───────────┘                    └───────────┘
      │
      │ TCP
      ▼
┌───────────┐
│  GitHub   │
│    API    │
└───────────┘
```

### Connection Types

| Color | Protocol |
|-------|----------|
| Blue | TCP |
| Green | UDP |
| Purple | Unix Socket |

### Known Endpoints

Known API endpoints are labeled:
- `api.anthropic.com` → "Anthropic API"
- `api.openai.com` → "OpenAI API"
- `github.com` → "GitHub"

## Filtering the Graph

### By Agent

Show processes from specific agents:

```
[x] Claude Code (12 processes)
[x] Cursor (8 processes)
[ ] Other (45 processes)
```

### By Status

Filter by process status:

```
[x] Running
[x] Exited (last hour)
[ ] Exited (older)
```

### By Process Name

Search to filter nodes:

```
🔍 node
```

Shows only processes matching "node".

## Real-Time Updates

The graph updates in real-time as processes spawn and exit:

### New Process Animation

- New nodes fade in
- Connection lines animate
- Brief highlight effect

### Process Exit Animation

- Node fades to gray
- Stays visible for configured time
- Then fades out

### Update Settings

```toml
[ui.graph]
# How long to show exited processes
exit_visible_duration = "5m"

# Animation duration
animation_duration = "300ms"

# Update throttle
update_interval = "100ms"
```

## Performance Mode

For large process trees, enable performance mode:

```toml
[ui.graph]
# Enable for >100 nodes
performance_mode = true

# Reduce physics simulation
simulation_ticks = 100

# Disable smooth animations
animations = false
```

### Large Graph Strategies

1. **Filter by agent** - Show only AI agents
2. **Collapse subtrees** - Hide deep branches
3. **Use tree layout** - More efficient than force
4. **Increase update interval** - Reduce redraw frequency

## Export Graph

### As Image

Export the current graph view:

```
Format: [PNG ▼] [SVG] [PDF]
Size: [1920x1080 ▼]
Include legend: [x]
```

### As Data

Export graph structure:

```bash
# JSON graph data
roea-cli graph export --format json --output graph.json

# DOT format (for Graphviz)
roea-cli graph export --format dot --output graph.dot
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `F` | Fit graph to view |
| `0` | Reset zoom |
| `+`/`-` | Zoom in/out |
| `L` | Cycle layout |
| `N` | Toggle network overlay |
| `R` | Refresh graph |
| `Escape` | Deselect all |

## Customization

### Node Appearance

```toml
[ui.graph.nodes]
# Agent node colors
claude_color = "#3b82f6"
cursor_color = "#8b5cf6"
default_color = "#6b7280"

# Size range
min_size = 20
max_size = 60
```

### Edge Appearance

```toml
[ui.graph.edges]
# Parent-child edges
parent_child_color = "#4b5563"
parent_child_width = 2

# Network edges
network_color = "#3b82f6"
network_width = 1
network_dash = "5,5"
```

## Next Steps

- [Dashboard](/features/dashboard) - Overview dashboard
- [Search & Filtering](/features/search) - Advanced filtering
- [Process Monitoring](/features/process-monitoring) - Process details
