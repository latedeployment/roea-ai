# File Access Monitoring

roea-ai tracks file system operations performed by AI coding agents, giving you visibility into which files are being read, written, and modified.

## Overview

When an AI agent runs, it typically accesses many files:
- Reading source code to understand context
- Writing new code or modifications
- Creating configuration files
- Accessing package manifests

roea-ai captures all these operations and correlates them with the originating agent.

## Tracked Operations

| Operation | Description | Icon |
|-----------|-------------|------|
| **Open** | File opened for reading/writing | 📂 |
| **Read** | File contents read | 📖 |
| **Write** | File contents written | ✏️ |
| **Create** | New file created | ➕ |
| **Delete** | File deleted | 🗑️ |
| **Rename** | File renamed/moved | 🔄 |

## File Access View

### Process Details Panel

When you select a process in the graph, the Details Panel shows recent file operations:

```
📁 File Access (Last 100)

📖 READ   src/main.rs                    10:32:15
✏️ WRITE  src/lib.rs                     10:32:18
📖 READ   Cargo.toml                     10:32:19
➕ CREATE src/new_module.rs              10:32:22
✏️ WRITE  src/new_module.rs              10:32:25
```

### File Timeline

The timeline view shows file operations over time:

```
10:30 ─────┬── READ package.json
           ├── READ src/index.ts
           └── READ tsconfig.json

10:32 ─────┬── WRITE src/index.ts
           ├── WRITE src/utils.ts
           └── CREATE src/new-file.ts

10:35 ─────┬── READ .env
           └── WRITE .env.local
```

## Filtering File Operations

### By File Path

Search for specific files or directories:

```
# All TypeScript files
*.ts

# Files in src directory
src/**

# Configuration files
*.json, *.yaml, *.toml

# Specific file
package.json
```

### By Operation Type

Filter by operation:
- **All** - Show all operations
- **Reads** - Only file reads
- **Writes** - Only file writes/creates
- **Deletes** - Only deletions

### By Agent

Filter to show only files accessed by a specific agent:
- Claude Code
- Cursor
- Aider
- All Agents

## File Classification

roea-ai automatically classifies files:

| Category | Patterns | Color |
|----------|----------|-------|
| **Source Code** | `.rs`, `.ts`, `.py`, `.go`, `.js` | Blue |
| **Configuration** | `.json`, `.yaml`, `.toml`, `.env` | Orange |
| **Documentation** | `.md`, `.txt`, `.rst` | Green |
| **Git Files** | `.git/*`, `.gitignore` | Purple |
| **Lock Files** | `*.lock`, `package-lock.json` | Gray |
| **Build Artifacts** | `target/`, `dist/`, `node_modules/` | Light Gray |

## Noise Filtering

roea-ai automatically filters noise to show relevant file operations:

### Default Filtered Paths

These paths are filtered by default:
- `/proc/*` - Linux process filesystem
- `/sys/*` - Linux sysfs
- `/dev/*` - Device files
- `node_modules/` - npm dependencies (can be enabled)
- `.git/objects/` - Git internal files
- `*.pyc`, `*.pyo` - Python bytecode
- `*.o`, `*.a` - Compiled objects

### Configure Filtering

In `roea.toml`:

```toml
[file_monitor]
# Add paths to filter
noise_patterns = [
    "/tmp/*",
    "*.log",
    ".cache/*",
]

# Remove default filters
include_node_modules = false
include_git_objects = false
```

## Sensitive File Detection

roea-ai highlights access to potentially sensitive files:

| File Type | Examples | Alert Level |
|-----------|----------|-------------|
| **Credentials** | `.env`, `credentials.json`, `*.pem` | High |
| **SSH Keys** | `~/.ssh/*`, `id_rsa` | High |
| **API Keys** | `*api_key*`, `*.key` | High |
| **Secrets** | `secrets.yaml`, `.secrets` | Medium |
| **Config** | `config.json`, `settings.yaml` | Low |

When sensitive files are accessed, they're highlighted in red in the UI.

## Export File Access Data

### JSON Export

```bash
roea-cli export file-ops --format json --output file_access.json
```

Output format:
```json
{
  "file_ops": [
    {
      "id": "uuid",
      "pid": 1234,
      "agent": "claude-code",
      "operation": "write",
      "path": "/home/user/project/src/main.rs",
      "size": 2048,
      "timestamp": "2026-01-13T10:32:18Z"
    }
  ]
}
```

### CSV Export

```bash
roea-cli export file-ops --format csv --output file_access.csv
```

### Filter During Export

```bash
# Only writes by Claude Code
roea-cli export file-ops \
  --filter "agent=claude-code AND operation=write" \
  --output claude_writes.json
```

## API Access

Query file operations via gRPC:

```bash
grpcurl -plaintext localhost:50051 roea.RoeaAgent/QueryFileOps
```

Or use the CLI:

```bash
# Recent file operations
roea-cli file-ops list --limit 100

# Filter by path
roea-cli file-ops list --path "src/*.rs"

# Filter by agent
roea-cli file-ops list --agent claude-code
```

## Use Cases

### Security Auditing

See exactly which files AI agents access:
```bash
roea-cli file-ops list \
  --agent "*" \
  --path "*.env,*.pem,*.key" \
  --since "1h"
```

### Understanding Agent Behavior

Track what files an agent reads for context:
```bash
roea-cli file-ops list \
  --agent claude-code \
  --operation read \
  --since "10m"
```

### Debugging File Issues

When files are unexpectedly modified:
```bash
roea-cli file-ops list \
  --path "path/to/file.ts" \
  --operation write
```

## Performance Considerations

File monitoring can generate high event volumes. For optimal performance:

1. **Enable noise filtering** - Filter irrelevant paths
2. **Use sampling** - For very high I/O workloads
3. **Adjust retention** - Keep file data for shorter periods

```toml
[file_monitor]
# Sample 1 in 10 events for high-volume paths
sampling_rate = 0.1
sampling_paths = ["node_modules/*", "*.log"]

[storage.retention]
# Keep file ops for less time
file_ops = "3d"
```

## Next Steps

- [Process Monitoring](/features/process-monitoring) - Track process execution
- [Network Tracking](/features/network-tracking) - Monitor network connections
- [Search & Filtering](/features/search) - Advanced search capabilities
