# Cursor IDE

roea-ai provides comprehensive monitoring for Cursor, the AI-powered code editor.

## Detection

Cursor is detected by:

| Method | Pattern |
|--------|---------|
| Process name | `Cursor`, `cursor` |
| Executable path | `Cursor.app`, `cursor/cursor` |
| Helper processes | `Cursor Helper`, `cursor-helper` |
| Network | `api.cursor.sh`, `api2.cursor.sh` |

## Process Tree

Typical Cursor process tree:

```
Cursor (main)
├── Cursor Helper (Renderer)
│   └── Extension processes
│       ├── rust-analyzer
│       ├── gopls
│       └── typescript-language-server
├── Cursor Helper (GPU)
├── Cursor Helper (Utility)
├── Cursor Helper (Network)
└── node (extension host)
    └── cursor-chat-extension
```

## Network Connections

Cursor makes connections to:

| Endpoint | Purpose |
|----------|---------|
| `api.cursor.sh` | Cursor AI API |
| `api2.cursor.sh` | Cursor API (v2) |
| `api.openai.com` | OpenAI API (if configured) |
| `api.anthropic.com` | Claude API (if configured) |
| `github.com` | Repository operations |
| `marketplace.visualstudio.com` | Extensions |

## Language Server Tracking

roea-ai tracks language servers spawned by Cursor:

| Language Server | Process Name |
|----------------|--------------|
| Rust | `rust-analyzer` |
| Go | `gopls` |
| TypeScript | `tsserver`, `typescript-language-server` |
| Python | `pylsp`, `pyright` |
| C/C++ | `clangd` |

## Common File Access Patterns

Cursor typically accesses:

| Category | Examples |
|----------|----------|
| Source code | All source files in workspace |
| Config | `.cursor/`, `settings.json` |
| Extensions | `~/.cursor/extensions/` |
| Cache | `~/.cursor/cache/` |

## Monitoring Tips

### Track AI Interactions

Monitor Cursor's AI API calls:

```bash
roea-cli query "
  SELECT remote_addr, remote_port, start_time, bytes_sent
  FROM connections
  WHERE pid IN (SELECT pid FROM processes WHERE agent_name = 'cursor')
    AND (remote_addr LIKE '%cursor.sh%'
         OR remote_addr LIKE '%openai%'
         OR remote_addr LIKE '%anthropic%')
  ORDER BY start_time DESC
  LIMIT 50
"
```

### Extension Activity

See which extensions are active:

```bash
roea-cli query "
  SELECT name, cmdline, start_time
  FROM processes
  WHERE parent_id IN (
    SELECT id FROM processes
    WHERE name LIKE '%Cursor%' AND cmdline LIKE '%extension%'
  )
"
```

### File Edit Tracking

Monitor files edited in Cursor:

```bash
roea-cli file-ops list \
  --agent cursor \
  --operation write \
  --path "*.ts,*.js,*.py,*.rs" \
  --since 1h
```

## Signature Configuration

Default Cursor signature:

```yaml
name: cursor
display_name: Cursor
icon: /icons/cursor.svg
process_patterns:
  - "^[Cc]ursor$"
  - "Cursor Helper"
  - "cursor-helper"
exe_path_patterns:
  - "Cursor\\.app/Contents/MacOS"
  - "cursor/cursor$"
  - "AppData.*[Cc]ursor"
  - "Local.*Programs.*Cursor"
network_endpoints:
  - "api.cursor.sh"
  - "api2.cursor.sh"
track_children: true
```

## Platform-Specific Notes

### macOS

Cursor.app bundle location:
```
/Applications/Cursor.app/Contents/MacOS/Cursor
```

Helper processes:
```
Cursor Helper (Renderer)
Cursor Helper (GPU)
Cursor Helper (Plugin)
```

### Windows

Typical installation:
```
C:\Users\<user>\AppData\Local\Programs\Cursor\Cursor.exe
```

### Linux

AppImage or installed location:
```
~/.local/bin/cursor
/opt/cursor/cursor
```

## Troubleshooting

### Cursor Not Detected

1. Check if Cursor is running:
   ```bash
   # macOS
   pgrep -f Cursor

   # Linux
   pgrep -f cursor
   ```

2. Verify process matches patterns:
   ```bash
   ps aux | grep -i cursor
   ```

3. Check for helper process names:
   ```bash
   ps aux | grep "Cursor Helper"
   ```

### Missing Language Servers

Language servers may not show as Cursor children if:
1. They're spawned by the extension host
2. The PID tracking was interrupted

Enable deeper tracking:
```toml
[agent_signatures.cursor]
track_grandchildren = true
```

### High Process Count

Cursor spawns many helper processes. Filter noise:
```bash
roea-cli processes list \
  --agent cursor \
  --filter "name NOT LIKE '%Helper%'"
```

## Next Steps

- [Process Monitoring](/features/process-monitoring) - Detailed process tracking
- [Network Tracking](/features/network-tracking) - API connection details
- [Agent Detection](/guide/agent-detection) - How detection works
