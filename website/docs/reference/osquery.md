# osquery Integration

roea-ai can integrate with osquery for enhanced system telemetry and advanced SQL queries.

## Overview

osquery provides:
- Cross-platform system introspection
- SQL interface for system data
- Battle-tested reliability
- Extensive table ecosystem

## Requirements

- osquery 5.0+ installed
- osquery daemon running (optional)

## Installation

### Ubuntu/Debian

```bash
# Add osquery repository
export OSQUERY_KEY=1484120AC4E9F8A1A577AEEE97A80C63C9D8B80B
apt-key adv --keyserver keyserver.ubuntu.com --recv-keys $OSQUERY_KEY
add-apt-repository 'deb [arch=amd64] https://pkg.osquery.io/deb deb main'

# Install
apt update && apt install osquery
```

### macOS

```bash
brew install osquery
```

### Windows

Download from [osquery releases](https://github.com/osquery/osquery/releases).

## Configuration

Enable osquery integration in `roea.toml`:

```toml
[osquery]
# Enable osquery integration
enabled = true

# osquery socket path
# Default: /var/osquery/osquery.em (Linux)
#          /var/osquery/osquery.em (macOS)
socket_path = "/var/osquery/osquery.em"

# Query timeout
timeout = "30s"

# Enable scheduled queries
scheduled_queries = true

# Query interval
query_interval = "60s"
```

## Built-in Queries

roea-ai runs these osquery queries:

### Process Information

```sql
-- Get detailed process info
SELECT
  pid, name, path, cmdline, cwd,
  uid, gid, euid, egid,
  start_time, parent
FROM processes
WHERE pid = ?;
```

### Open Files

```sql
-- Files opened by process
SELECT
  p.pid, p.name, pof.fd, pof.path
FROM processes p
JOIN process_open_files pof ON p.pid = pof.pid
WHERE p.pid = ?;
```

### Network Connections

```sql
-- Process connections
SELECT
  p.pid, p.name,
  ps.local_address, ps.local_port,
  ps.remote_address, ps.remote_port,
  ps.protocol, ps.state
FROM processes p
JOIN process_open_sockets ps ON p.pid = ps.pid
WHERE p.pid = ?;
```

### Process Tree

```sql
-- Full process tree
WITH RECURSIVE tree AS (
  SELECT pid, parent, name, 0 as depth
  FROM processes WHERE pid = ?
  UNION ALL
  SELECT p.pid, p.parent, p.name, t.depth + 1
  FROM processes p
  JOIN tree t ON p.parent = t.pid
  WHERE t.depth < 10
)
SELECT * FROM tree;
```

## Custom Queries

Run custom osquery queries:

```bash
# Via CLI
roea-cli osquery "SELECT * FROM processes WHERE name LIKE '%claude%'"

# Via gRPC
grpcurl -plaintext \
  -d '{"query": "SELECT * FROM processes LIMIT 10"}' \
  localhost:50051 roea.RoeaAgent/OsqueryQuery
```

## Scheduled Queries

Configure scheduled queries for continuous monitoring:

```toml
[[osquery.schedules]]
name = "ai_agent_processes"
query = """
  SELECT pid, name, cmdline, start_time
  FROM processes
  WHERE name IN ('claude', 'cursor', 'aider')
"""
interval = "30s"

[[osquery.schedules]]
name = "suspicious_connections"
query = """
  SELECT p.name, ps.remote_address, ps.remote_port
  FROM processes p
  JOIN process_open_sockets ps ON p.pid = ps.pid
  WHERE ps.remote_address NOT IN ('127.0.0.1', '::1')
    AND ps.remote_port NOT IN (80, 443)
"""
interval = "60s"
```

## Useful osquery Tables

### Process Tables

| Table | Description |
|-------|-------------|
| `processes` | Running processes |
| `process_open_files` | Open file handles |
| `process_open_sockets` | Network sockets |
| `process_memory_map` | Memory mappings |
| `process_envs` | Environment variables |

### System Tables

| Table | Description |
|-------|-------------|
| `listening_ports` | Open listening ports |
| `logged_in_users` | Currently logged in users |
| `mounts` | Mounted filesystems |
| `system_info` | System information |

### Network Tables

| Table | Description |
|-------|-------------|
| `interface_addresses` | Network interfaces |
| `routes` | Routing table |
| `arp_cache` | ARP cache |
| `dns_resolvers` | DNS configuration |

## Performance Considerations

osquery can be resource-intensive. Optimize:

```toml
[osquery]
# Limit concurrent queries
max_concurrent = 2

# Cache query results
cache_enabled = true
cache_ttl = "30s"

# Increase timeout for complex queries
timeout = "60s"
```

## Troubleshooting

### Connection Failed

```bash
# Check if osqueryd is running
systemctl status osqueryd

# Check socket exists
ls -la /var/osquery/osquery.em

# Check permissions
sudo -u roea ls /var/osquery/osquery.em
```

### Query Timeout

For slow queries:
1. Add appropriate WHERE clauses
2. Limit result sets
3. Increase timeout in config

### Permission Denied

osquery may need root for some tables:
```bash
# Run osqueryd as root
sudo osqueryd --config_path=/etc/osquery/osquery.conf
```

## See Also

- [How It Works](/guide/how-it-works) - Architecture overview
- [Process Monitoring](/features/process-monitoring) - Process details
- [osquery Documentation](https://osquery.io/docs) - Official docs
