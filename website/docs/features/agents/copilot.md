# VS Code Copilot

roea-ai monitors GitHub Copilot running within VS Code and other editors.

## Detection

GitHub Copilot is detected by:

| Method | Pattern |
|--------|---------|
| Process name | `copilot`, `github.copilot` |
| Extension path | `extensions/*copilot*` |
| Network | `copilot-proxy.githubusercontent.com` |

## Process Tree

Copilot runs as a VS Code extension:

```
code (VS Code)
└── node (extension host)
    ├── github.copilot
    ├── github.copilot-chat
    └── other extensions...
```

## Network Connections

Copilot makes connections to:

| Endpoint | Purpose |
|----------|---------|
| `copilot-proxy.githubusercontent.com` | Copilot API |
| `api.github.com` | GitHub API |
| `github.com` | Authentication |
| `api.githubcopilot.com` | Copilot services |

## Monitoring Tips

### Track Completions

Monitor Copilot API calls:

```bash
roea-cli query "
  SELECT start_time, bytes_sent, bytes_recv
  FROM connections
  WHERE remote_addr LIKE '%copilot%' OR remote_addr LIKE '%githubcopilot%'
  ORDER BY start_time DESC
"
```

### Copilot Chat Usage

Track Copilot Chat interactions:

```bash
roea-cli query "
  SELECT COUNT(*) as chats, DATE(start_time) as date
  FROM connections
  WHERE remote_addr LIKE '%copilot%'
    AND bytes_sent > 1000  -- Chat messages are larger
  GROUP BY date
"
```

## Signature Configuration

```yaml
name: copilot
display_name: GitHub Copilot
icon: /icons/copilot.svg
process_patterns:
  - "copilot"
  - "github\\.copilot"
exe_path_patterns:
  - "extensions.*copilot"
  - "github\\.copilot.*dist"
network_endpoints:
  - "copilot-proxy.githubusercontent.com"
  - "api.githubcopilot.com"
  - "api.github.com"
track_children: false
parent_hints:
  - "code"
  - "Code Helper"
  - "electron"
```

## Notes

- Copilot runs within VS Code's extension host
- It doesn't spawn child processes directly
- Network monitoring is the best way to track activity
- Copilot Chat sends larger payloads than completions

## Next Steps

- [VS Code Extensions](/features/process-monitoring#extensions) - Extension tracking
- [Network Tracking](/features/network-tracking) - API monitoring
