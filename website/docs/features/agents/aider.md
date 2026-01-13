# Aider

roea-ai monitors Aider, the AI pair programming CLI tool.

## Detection

Aider is detected by:

| Method | Pattern |
|--------|---------|
| Process name | `aider` |
| Command line | `aider`, `--model`, `--edit-format` |
| Network | `api.openai.com`, `api.anthropic.com` |

## Process Tree

Typical Aider process tree:

```
Terminal (bash/zsh)
└── python
    └── aider
        ├── git (commits)
        └── shell commands (if enabled)
```

## Network Connections

Aider connects to:

| Endpoint | Purpose |
|----------|---------|
| `api.openai.com` | OpenAI API (GPT models) |
| `api.anthropic.com` | Anthropic API (Claude models) |
| `github.com` | Repository operations |

## Command Line Flags

Common Aider invocations:

```bash
# With GPT-4
aider --model gpt-4

# With Claude
aider --model claude-3-opus-20240229

# Architect mode
aider --architect

# With specific files
aider src/main.py src/utils.py
```

## Monitoring Tips

### Track Model Usage

See which AI models Aider uses:

```bash
roea-cli query "
  SELECT cmdline, start_time
  FROM processes
  WHERE agent_name = 'aider'
  ORDER BY start_time DESC
"
```

### Git Commits by Aider

Track commits Aider makes:

```bash
roea-cli query "
  SELECT p.cmdline, p.start_time
  FROM processes p
  WHERE p.name = 'git'
    AND p.parent_id IN (
      SELECT id FROM processes WHERE agent_name = 'aider'
    )
    AND p.cmdline LIKE '%commit%'
"
```

### API Cost Estimation

Estimate API usage:

```bash
roea-cli query "
  SELECT
    remote_addr,
    COUNT(*) as calls,
    SUM(bytes_sent) as request_bytes,
    SUM(bytes_recv) as response_bytes
  FROM connections
  WHERE pid IN (SELECT pid FROM processes WHERE agent_name = 'aider')
    AND (remote_addr LIKE '%openai%' OR remote_addr LIKE '%anthropic%')
  GROUP BY remote_addr
"
```

## Signature Configuration

```yaml
name: aider
display_name: Aider
icon: /icons/aider.svg
process_patterns:
  - "^aider$"
  - "aider-chat"
command_patterns:
  - "aider"
  - "--model"
  - "--edit-format"
  - "--architect"
exe_path_patterns:
  - "bin/aider"
  - "site-packages/aider"
network_endpoints:
  - "api.openai.com"
  - "api.anthropic.com"
track_children: true
parent_hints:
  - "python"
  - "python3"
  - "bash"
  - "zsh"
```

## Notes

- Aider runs as a Python process
- It may appear as `python` with `aider` in the command line
- Aider makes frequent git commits, which are tracked as child processes
- Watch/auto-commit mode creates continuous activity

## Troubleshooting

### Aider Shows as Python

If Aider appears as generic `python`:

1. Check the command line pattern matching is enabled
2. Look for `aider` in the cmdline field:
   ```bash
   roea-cli query "SELECT * FROM processes WHERE cmdline LIKE '%aider%'"
   ```

### Missing Git Operations

Git operations may not link to Aider if:
1. Git runs from a separate shell
2. Parent process tracking is interrupted

## Next Steps

- [Process Monitoring](/features/process-monitoring) - Process tracking
- [Network Tracking](/features/network-tracking) - API monitoring
- [Agent Detection](/guide/agent-detection) - Detection configuration
