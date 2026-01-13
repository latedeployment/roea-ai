# Data Storage

roea-ai stores all telemetry data locally using DuckDB, an embedded analytical database. This page covers storage configuration, retention policies, and data management.

## Storage Overview

All data is stored in a single DuckDB database file:

| Platform | Default Location |
|----------|-----------------|
| **Linux** | `~/.local/share/roea/data.duckdb` |
| **macOS** | `~/Library/Application Support/roea/data.duckdb` |
| **Windows** | `%APPDATA%\roea\data.duckdb` |

## Database Schema

### processes table

Stores process lifecycle events:

```sql
CREATE TABLE processes (
    id UUID PRIMARY KEY,
    pid INTEGER NOT NULL,
    ppid INTEGER,
    name TEXT NOT NULL,
    cmdline TEXT,
    exe_path TEXT,
    cwd TEXT,
    user TEXT,
    start_time TIMESTAMP NOT NULL,
    exit_time TIMESTAMP,
    exit_code INTEGER,
    agent_name TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### connections table

Stores network connection events:

```sql
CREATE TABLE connections (
    id UUID PRIMARY KEY,
    pid INTEGER NOT NULL,
    process_id UUID REFERENCES processes(id),
    protocol TEXT NOT NULL,  -- 'tcp', 'udp', 'unix'
    local_addr TEXT,
    local_port INTEGER,
    remote_addr TEXT,
    remote_port INTEGER,
    state TEXT,              -- 'connecting', 'established', 'closed'
    bytes_sent BIGINT,
    bytes_recv BIGINT,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### file_ops table

Stores file system operations:

```sql
CREATE TABLE file_ops (
    id UUID PRIMARY KEY,
    pid INTEGER NOT NULL,
    process_id UUID REFERENCES processes(id),
    operation TEXT NOT NULL,  -- 'open', 'read', 'write', 'delete', 'rename'
    path TEXT NOT NULL,
    size BIGINT,
    timestamp TIMESTAMP NOT NULL,
    success BOOLEAN,
    created_at TIMESTAMP DEFAULT NOW()
);
```

## Storage Configuration

Configure storage in `roea.toml`:

```toml
[storage]
# Database file path (default: platform-specific)
path = "~/.local/share/roea/data.duckdb"

# Maximum database size (default: unlimited)
max_size = "10GB"

# Enable WAL mode for better write performance
wal_mode = true

# Checkpoint interval in seconds
checkpoint_interval = 300
```

## Retention Policies

Configure data retention to manage disk usage:

```toml
[storage.retention]
# Keep process data for 30 days
processes = "30d"

# Keep connection data for 7 days
connections = "7d"

# Keep file operation data for 7 days
file_ops = "7d"

# Run cleanup every hour
cleanup_interval = "1h"
```

### Retention Syntax

| Format | Example | Description |
|--------|---------|-------------|
| Days | `30d` | 30 days |
| Hours | `168h` | 168 hours (7 days) |
| Minutes | `10080m` | 10080 minutes (7 days) |
| Forever | `forever` | Never delete |

## Querying Data

### Using roea-cli

```bash
# List recent processes
roea-cli query "SELECT * FROM processes ORDER BY start_time DESC LIMIT 10"

# Find all Claude Code sessions
roea-cli query "SELECT * FROM processes WHERE agent_name = 'claude-code'"

# Network connections to Anthropic API
roea-cli query "SELECT * FROM connections WHERE remote_addr LIKE '%anthropic%'"
```

### Direct DuckDB Access

```bash
# Open database directly
duckdb ~/.local/share/roea/data.duckdb

# Run queries
SELECT COUNT(*) FROM processes WHERE agent_name IS NOT NULL;
```

### SQL Examples

**Process activity by agent:**
```sql
SELECT
    agent_name,
    COUNT(*) as process_count,
    MIN(start_time) as first_seen,
    MAX(start_time) as last_seen
FROM processes
WHERE agent_name IS NOT NULL
GROUP BY agent_name
ORDER BY process_count DESC;
```

**Network connections by endpoint:**
```sql
SELECT
    remote_addr,
    remote_port,
    COUNT(*) as connection_count,
    SUM(bytes_sent) as total_bytes_sent,
    SUM(bytes_recv) as total_bytes_recv
FROM connections
GROUP BY remote_addr, remote_port
ORDER BY connection_count DESC
LIMIT 20;
```

**Files touched by AI agents:**
```sql
SELECT
    p.agent_name,
    f.path,
    f.operation,
    f.timestamp
FROM file_ops f
JOIN processes p ON f.process_id = p.id
WHERE p.agent_name IS NOT NULL
ORDER BY f.timestamp DESC
LIMIT 100;
```

## Data Export

### Export to JSON

```bash
# Export all processes
roea-cli export --format json --output processes.json processes

# Export with filter
roea-cli export --format json --filter "agent_name = 'claude-code'" processes
```

### Export to CSV

```bash
# Export connections
roea-cli export --format csv --output connections.csv connections

# Export file operations
roea-cli export --format csv --output file_ops.csv file_ops
```

### Export to Parquet

```bash
# Export for analytics
roea-cli export --format parquet --output data.parquet processes
```

## Backup and Restore

### Manual Backup

```bash
# Stop the daemon first
systemctl --user stop roea-agent

# Copy database file
cp ~/.local/share/roea/data.duckdb ~/backup/roea-$(date +%Y%m%d).duckdb

# Restart daemon
systemctl --user start roea-agent
```

### Automated Backup

Configure automatic backups in `roea.toml`:

```toml
[storage.backup]
# Enable automatic backups
enabled = true

# Backup directory
path = "~/backup/roea"

# Backup frequency
interval = "24h"

# Keep this many backups
keep = 7
```

### Restore from Backup

```bash
# Stop daemon
systemctl --user stop roea-agent

# Replace database
cp ~/backup/roea-20260113.duckdb ~/.local/share/roea/data.duckdb

# Start daemon
systemctl --user start roea-agent
```

## Performance Tuning

### For High-Volume Systems

```toml
[storage]
# Use larger page cache
page_cache_size = "512MB"

# Increase checkpoint interval
checkpoint_interval = 600

# Enable parallel writes
parallel_writes = true
```

### For Limited Disk Space

```toml
[storage]
# Limit database size
max_size = "1GB"

# Aggressive retention
[storage.retention]
processes = "7d"
connections = "1d"
file_ops = "1d"
cleanup_interval = "30m"
```

## Troubleshooting

### Database Locked

If you see "database is locked" errors:

```bash
# Check for running processes
lsof ~/.local/share/roea/data.duckdb

# Force checkpoint
roea-cli storage checkpoint
```

### Database Corruption

If the database appears corrupted:

```bash
# Export to recover data
duckdb ~/.local/share/roea/data.duckdb \
  "EXPORT DATABASE '~/roea-export' (FORMAT PARQUET)"

# Delete and recreate
rm ~/.local/share/roea/data.duckdb
systemctl --user restart roea-agent

# Re-import if needed
roea-cli import ~/roea-export
```

### Disk Space Issues

Monitor disk usage:

```bash
# Check database size
du -h ~/.local/share/roea/data.duckdb

# Run manual cleanup
roea-cli storage cleanup --older-than 7d
```

## Next Steps

- [Configuration](/reference/configuration) - Full configuration reference
- [Environment Variables](/reference/environment) - Environment configuration
- [Troubleshooting](/reference/troubleshooting) - Common issues
