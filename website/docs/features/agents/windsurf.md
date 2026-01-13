# Windsurf

roea-ai monitors Windsurf (by Codeium), the AI-powered code editor.

## Detection

Windsurf is detected by:

| Method | Pattern |
|--------|---------|
| Process name | `windsurf`, `Windsurf` |
| Executable path | `Windsurf.app`, `windsurf/windsurf` |
| Network | `api.codeium.com` |

## Process Tree

Typical Windsurf process tree:

```
Windsurf (main)
├── Windsurf Helper (Renderer)
├── Windsurf Helper (GPU)
├── Windsurf Helper (Utility)
└── node (extension host)
    └── codeium extension
```

## Network Connections

Windsurf connects to:

| Endpoint | Purpose |
|----------|---------|
| `api.codeium.com` | Codeium AI API |
| `server.codeium.com` | Codeium services |
| `github.com` | Repository operations |

## Signature Configuration

```yaml
name: windsurf
display_name: Windsurf
icon: /icons/windsurf.svg
process_patterns:
  - "^[Ww]indsurf$"
  - "Windsurf Helper"
exe_path_patterns:
  - "Windsurf\\.app"
  - "windsurf/windsurf"
  - "Programs.*Windsurf"
network_endpoints:
  - "api.codeium.com"
  - "server.codeium.com"
track_children: true
```

## Monitoring Tips

### Track Codeium API Usage

```bash
roea-cli query "
  SELECT COUNT(*) as calls, SUM(bytes_sent) as bytes
  FROM connections
  WHERE remote_addr LIKE '%codeium%'
"
```

## Next Steps

- [Process Monitoring](/features/process-monitoring) - Process tracking
- [Network Tracking](/features/network-tracking) - API monitoring
