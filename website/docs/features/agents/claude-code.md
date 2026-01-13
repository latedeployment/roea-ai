# Claude Code

roea-ai provides comprehensive monitoring for Claude Code, Anthropic's AI coding assistant.

## Detection

Claude Code is detected by:

| Method | Pattern |
|--------|---------|
| Process name | `claude`, `claude-code` |
| Command line | `claude chat`, `claude code` |
| Network | `api.anthropic.com` |

## Process Tree

Typical Claude Code process tree:

```
Terminal (bash/zsh/fish)
└── claude
    ├── bash (tool execution)
    │   ├── npm run ...
    │   ├── cargo build
    │   └── git commit
    ├── node (MCP servers)
    └── python (analysis scripts)
```

## Network Connections

Claude Code makes connections to:

| Endpoint | Purpose |
|----------|---------|
| `api.anthropic.com` | Claude API |
| `github.com` | Repository operations |
| `api.github.com` | GitHub API |
| `registry.npmjs.org` | npm packages |
| `pypi.org` | Python packages |

## Common File Access Patterns

Claude Code typically accesses:

| Category | Examples |
|----------|----------|
| Source code | `*.ts`, `*.py`, `*.rs`, `*.go` |
| Config | `package.json`, `Cargo.toml`, `.env` |
| Documentation | `README.md`, `*.md` |
| Git | `.git/config`, `.gitignore` |

## Monitoring Tips

### Track Tool Usage

Monitor which tools Claude Code invokes:

```bash
roea-cli query "
  SELECT name, cmdline, start_time
  FROM processes
  WHERE parent_id IN (
    SELECT id FROM processes WHERE agent_name = 'claude-code'
  )
  ORDER BY start_time DESC
"
```

### API Call Volume

Track API usage:

```bash
roea-cli query "
  SELECT COUNT(*) as calls, DATE(start_time) as date
  FROM connections
  WHERE remote_addr LIKE '%anthropic%'
    AND pid IN (SELECT pid FROM processes WHERE agent_name = 'claude-code')
  GROUP BY date
"
```

### File Modification Audit

See what files Claude Code modifies:

```bash
roea-cli file-ops list \
  --agent claude-code \
  --operation write \
  --since 1h
```

## Signature Configuration

Default Claude Code signature:

```yaml
name: claude-code
display_name: Claude Code
icon: /icons/claude.svg
process_patterns:
  - "^claude$"
  - "claude-code"
command_patterns:
  - "claude\\s+(chat|code|api)"
  - "--api-key"
  - "--model"
network_endpoints:
  - "api.anthropic.com"
  - "console.anthropic.com"
track_children: true
parent_hints:
  - "bash"
  - "zsh"
  - "fish"
  - "Terminal"
  - "iTerm"
  - "alacritty"
  - "kitty"
```

## Troubleshooting

### Claude Code Not Detected

1. Check if Claude Code is running:
   ```bash
   ps aux | grep claude
   ```

2. Verify process name matches patterns:
   ```bash
   # Get full command line
   cat /proc/$(pgrep claude)/cmdline | tr '\0' ' '
   ```

3. Check roea-agent logs:
   ```bash
   journalctl -u roea-agent | grep claude
   ```

### Missing Child Processes

If spawned processes aren't tracked:

1. Ensure `track_children: true` is set
2. Check parent PID matches Claude Code
3. Verify child process is not filtered by noise rules

## Integration with Claude Code

### View Monitoring in Claude Code

You can ask Claude Code to check what roea-ai is tracking:

```
Claude, can you check what processes roea-ai sees me running?
```

### Export Session Report

After a Claude Code session:

```bash
roea-cli export session \
  --agent claude-code \
  --since "session-start" \
  --format markdown \
  --output claude-session.md
```

## Next Steps

- [Process Monitoring](/features/process-monitoring) - Detailed process tracking
- [Network Tracking](/features/network-tracking) - API connection details
- [Agent Detection](/guide/agent-detection) - How detection works
