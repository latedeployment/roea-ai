# Storage Settings

Complete reference for roea-ai storage configuration.

## Database Configuration

```toml
[storage]
# Database file path
# Default: Platform-specific application data directory
path = "~/.local/share/roea/data.duckdb"

# Maximum database size
# Supported units: KB, MB, GB
# Default: unlimited
max_size = "10GB"

# Enable Write-Ahead Logging for better write performance
# Default: true
wal_mode = true

# Checkpoint interval in seconds
# Writes WAL to main database file
# Default: 300
checkpoint_interval = 300

# Page cache size for query performance
# Default: 256MB
page_cache_size = "256MB"

# Enable parallel writes (experimental)
# Default: false
parallel_writes = false
```

## Retention Configuration

```toml
[storage.retention]
# How long to keep process data
# Supported units: m (minutes), h (hours), d (days), w (weeks)
# Default: 30d
processes = "30d"

# How long to keep connection data
# Default: 7d
connections = "7d"

# How long to keep file operation data
# Default: 7d
file_ops = "7d"

# How often to run cleanup
# Default: 1h
cleanup_interval = "1h"

# Keep all data (disable retention)
# Default: false
keep_forever = false
```

## Backup Configuration

```toml
[storage.backup]
# Enable automatic backups
# Default: false
enabled = true

# Backup directory
# Default: Platform-specific backup directory
path = "~/backup/roea"

# Backup frequency
# Default: 24h
interval = "24h"

# Number of backups to keep
# Default: 7
keep = 7

# Compression format
# Options: none, gzip, zstd
# Default: zstd
compression = "zstd"
```

## Performance Tuning

```toml
[storage.performance]
# Batch size for bulk inserts
# Default: 1000
batch_size = 1000

# Flush interval for batched writes
# Default: 1s
flush_interval = "1s"

# Index configuration
# Default: true
create_indexes = true

# Vacuum schedule (reclaim space)
# Default: weekly
vacuum_schedule = "weekly"
```

## Platform Defaults

### Linux

```
Database: ~/.local/share/roea/data.duckdb
Backup:   ~/.local/share/roea/backups/
Logs:     ~/.local/share/roea/logs/
```

### macOS

```
Database: ~/Library/Application Support/roea/data.duckdb
Backup:   ~/Library/Application Support/roea/backups/
Logs:     ~/Library/Logs/roea/
```

### Windows

```
Database: %APPDATA%\roea\data.duckdb
Backup:   %APPDATA%\roea\backups\
Logs:     %APPDATA%\roea\logs\
```

## CLI Commands

```bash
# Check storage status
roea-cli storage status

# Run manual cleanup
roea-cli storage cleanup --older-than 7d

# Create backup
roea-cli storage backup --output ./backup.duckdb

# Restore from backup
roea-cli storage restore --input ./backup.duckdb

# Compact database
roea-cli storage vacuum

# Export all data
roea-cli storage export --format parquet --output ./export/
```

## See Also

- [Data Storage Guide](/guide/storage) - Storage concepts
- [Configuration](/reference/configuration) - Full config reference
- [Environment Variables](/reference/environment) - Env var reference
