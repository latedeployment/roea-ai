# Search & Filtering

roea-ai provides powerful search and filtering capabilities to help you find specific processes, connections, and file operations across your monitoring data.

## Quick Search

The search bar at the top of the UI provides instant filtering:

```
┌──────────────────────────────────────────────┐
│ 🔍 Search processes, connections, files...   │
└──────────────────────────────────────────────┘
```

### Search Targets

Quick search matches against:
- **Process name** - `claude`, `node`, `python`
- **PID** - `1234`, `5678`
- **Command line** - `--api-key`, `serve`
- **Executable path** - `/usr/bin/node`
- **Network addresses** - `api.anthropic.com`
- **File paths** - `src/main.rs`

### Search Syntax

| Syntax | Example | Description |
|--------|---------|-------------|
| Plain text | `claude` | Matches anywhere |
| Quoted | `"claude code"` | Exact phrase |
| Prefix | `name:claude` | Field-specific |
| Wildcard | `*.ts` | Glob patterns |
| Regex | `/^node/` | Regular expression |
| Negation | `-test` | Exclude matches |

## Advanced Filters

Click the filter button (⚙️) to access advanced filtering:

### Process Filters

```
Agent Type:      [x] Claude Code
                 [x] Cursor
                 [x] Aider
                 [ ] Other

Status:          [x] Running
                 [x] Exited

PID Range:       From: [     ] To: [     ]

Start Time:      From: [         ] To: [         ]
```

### Connection Filters

```
Protocol:        [x] TCP
                 [x] UDP
                 [ ] Unix Socket

State:           [x] Established
                 [x] Connecting
                 [ ] Closed

Port Range:      From: [     ] To: [     ]

Endpoint:        [ Known APIs        ▼]
```

### File Operation Filters

```
Operation:       [x] Read
                 [x] Write
                 [ ] Delete
                 [ ] Create

File Type:       [x] Source Code
                 [x] Configuration
                 [ ] Documentation
                 [ ] Build Artifacts

Path Pattern:    [                    ]
```

## Query DSL

For complex queries, use the query DSL:

### Process Queries

```
# Find Claude Code processes
agent:claude-code

# Running processes only
status:running

# Processes started in last hour
start_time:>-1h

# Specific PID range
pid:1000..2000

# Combined query
agent:cursor AND status:running AND start_time:>-1h
```

### Connection Queries

```
# API connections
remote:*.anthropic.com OR remote:*.openai.com

# Established TCP connections
protocol:tcp AND state:established

# High port connections
remote_port:>1024

# Specific process connections
pid:1234 AND protocol:tcp
```

### File Queries

```
# Source code modifications
path:*.rs AND operation:write

# All TypeScript reads
path:*.ts AND operation:read

# Changes in src directory
path:src/** AND (operation:write OR operation:create)

# Sensitive file access
path:*.env OR path:*.key OR path:*.pem
```

## Filter Combinations

### Boolean Operators

| Operator | Example | Description |
|----------|---------|-------------|
| `AND` | `a AND b` | Both conditions |
| `OR` | `a OR b` | Either condition |
| `NOT` | `NOT a` | Exclude condition |
| `()` | `(a OR b) AND c` | Grouping |

### Time Filters

| Syntax | Example | Description |
|--------|---------|-------------|
| Relative | `>-1h` | Last hour |
| Relative | `>-7d` | Last 7 days |
| Absolute | `>2026-01-13` | After date |
| Range | `2026-01-13..2026-01-14` | Date range |

### Numeric Filters

| Syntax | Example | Description |
|--------|---------|-------------|
| Equals | `pid:1234` | Exact match |
| Greater | `port:>8000` | Greater than |
| Less | `port:<1024` | Less than |
| Range | `pid:1000..2000` | Inclusive range |

## Saved Searches

Save frequently used searches for quick access:

### Create Saved Search

1. Enter your search query
2. Click "Save Search" (⭐)
3. Name your search
4. Optionally add description

### Manage Saved Searches

Access saved searches from the dropdown:

```
⭐ Saved Searches
├── Claude Code Activity
├── API Connections
├── Source File Changes
└── + Create New...
```

### Default Saved Searches

roea-ai includes some default saved searches:

| Name | Query |
|------|-------|
| **All AI Agents** | `agent:*` |
| **Active Processes** | `status:running AND agent:*` |
| **API Connections** | `remote:*.anthropic.com OR remote:*.openai.com OR remote:*.cursor.sh` |
| **Source Changes** | `operation:write AND path:src/**` |
| **Recent Activity** | `start_time:>-1h` |

## Export Search Results

Export filtered data for external analysis:

### Export Options

```
Format:     [JSON ▼]  [CSV] [Parquet]

Include:    [x] Processes
            [x] Connections
            [x] File Operations

Columns:    [Select columns...]
```

### CLI Export

```bash
# Export search results
roea-cli search "agent:claude-code AND start_time:>-1h" \
  --format json \
  --output results.json

# Export with specific fields
roea-cli search "protocol:tcp" \
  --format csv \
  --columns pid,remote_addr,remote_port,state \
  --output connections.csv
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus search bar |
| `Escape` | Clear search |
| `Enter` | Execute search |
| `Ctrl+S` | Save current search |
| `Ctrl+E` | Export results |
| `↑/↓` | Navigate search history |

## Search Performance

For large datasets, optimize your searches:

### Tips for Fast Searches

1. **Use specific fields** - `agent:claude-code` is faster than plain `claude-code`
2. **Add time filters** - Limit the search range
3. **Avoid leading wildcards** - `*.ts` is fine, `*claude*` is slower
4. **Index hints** - High-cardinality fields are indexed

### Search Limits

```toml
[search]
# Maximum results returned
max_results = 10000

# Search timeout
timeout = "30s"

# Enable search cache
cache_enabled = true
cache_ttl = "5m"
```

## Next Steps

- [Process Monitoring](/features/process-monitoring) - Understanding process data
- [Network Tracking](/features/network-tracking) - Network connection details
- [File Access](/features/file-access) - File operation monitoring
