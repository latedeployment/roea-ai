# Troubleshooting

Common issues and solutions for roea-ai.

## Installation Issues

### "Command not found"

After installing, `roea-agent` command not found:

```bash
# Check if binary exists
ls -la /usr/local/bin/roea-agent

# Add to PATH if installed elsewhere
export PATH=$PATH:/path/to/roea

# Reload shell
source ~/.bashrc
```

### Permission Denied (Linux)

```bash
# Make binary executable
chmod +x /usr/local/bin/roea-agent

# For eBPF, grant capabilities
sudo setcap cap_bpf+ep /usr/local/bin/roea-agent
```

### macOS Security Block

If macOS blocks the app:

1. Open System Preferences → Security & Privacy
2. Click "Allow Anyway" for roea-ai
3. Or run: `xattr -dr com.apple.quarantine /Applications/roea-ai.app`

## Startup Issues

### "Failed to bind to port"

Another process is using the port:

```bash
# Find what's using the port
lsof -i :50051

# Change roea-ai port
export ROEA_GRPC_ADDR=127.0.0.1:50052
roea-agent
```

### "Database locked"

Database is in use by another process:

```bash
# Find processes using the database
lsof ~/.local/share/roea/data.duckdb

# Kill stale processes
pkill -f roea-agent

# Remove lock file if exists
rm ~/.local/share/roea/data.duckdb.wal
```

### "eBPF not available"

On Linux, if eBPF fails:

```bash
# Check kernel version (need 4.18+)
uname -r

# Check BTF support
ls /sys/kernel/btf/vmlinux

# Check capabilities
getcap /usr/local/bin/roea-agent

# Run as root as fallback
sudo roea-agent
```

## Agent Detection Issues

### Agent Not Detected

If your AI agent isn't showing:

1. **Verify agent is running:**
   ```bash
   ps aux | grep -E "(claude|cursor|aider)"
   ```

2. **Check signature patterns:**
   ```bash
   roea-cli signatures list
   roea-cli signatures test --process-name "claude"
   ```

3. **Enable debug logging:**
   ```bash
   RUST_LOG=roea_agent::signatures=debug roea-agent
   ```

4. **Add custom signature** if needed (see [Agent Detection](/guide/agent-detection))

### False Positives

If non-agent processes are detected:

1. **Check which signature matched:**
   ```bash
   roea-cli processes show --pid 1234 --verbose
   ```

2. **Make patterns more specific** in signature YAML

3. **Add exclusion pattern:**
   ```toml
   [[agent_signatures.claude-code]]
   exclude_patterns = ["claude-test", "claude-mock"]
   ```

## UI Issues

### UI Won't Connect

If the desktop UI can't connect to the daemon:

1. **Check daemon is running:**
   ```bash
   systemctl --user status roea-agent
   # or
   pgrep roea-agent
   ```

2. **Check gRPC address:**
   ```bash
   roea-cli status
   ```

3. **Verify firewall:**
   ```bash
   # Check if port is open
   nc -zv localhost 50051
   ```

### Graph Not Rendering

If the process graph is blank:

1. **Check for JavaScript errors** (open DevTools with F12)

2. **Verify data is available:**
   ```bash
   roea-cli processes list --limit 5
   ```

3. **Reset UI state:**
   - Clear browser cache if using web UI
   - Delete `~/.config/roea-ui/` for desktop app

### Slow UI Performance

For large process trees:

1. **Enable performance mode** in settings
2. **Filter by agent** to reduce nodes
3. **Reduce update frequency:**
   ```toml
   [ui]
   update_interval = "500ms"
   ```

## Data Issues

### Missing Data

If processes/connections are missing:

1. **Check retention settings:**
   ```bash
   roea-cli config show | grep retention
   ```

2. **Verify data is being collected:**
   ```bash
   roea-cli query "SELECT COUNT(*) FROM processes WHERE start_time > datetime('now', '-1 hour')"
   ```

3. **Check for errors:**
   ```bash
   journalctl -u roea-agent | grep -i error
   ```

### High Disk Usage

If database grows too large:

1. **Check current size:**
   ```bash
   du -h ~/.local/share/roea/data.duckdb
   ```

2. **Run cleanup:**
   ```bash
   roea-cli storage cleanup --older-than 7d
   ```

3. **Adjust retention:**
   ```toml
   [storage.retention]
   processes = "7d"
   connections = "3d"
   file_ops = "3d"
   ```

4. **Vacuum database:**
   ```bash
   roea-cli storage vacuum
   ```

## Performance Issues

### High CPU Usage

If roea-agent uses too much CPU:

1. **Check what's consuming CPU:**
   ```bash
   top -p $(pgrep roea-agent)
   ```

2. **Reduce polling frequency:**
   ```toml
   [monitor]
   process_poll_interval = "2000ms"
   network_poll_interval = "5000ms"
   ```

3. **Disable unused monitors:**
   ```toml
   [monitor.file]
   enabled = false
   ```

### High Memory Usage

If memory grows:

1. **Check memory:**
   ```bash
   ps -o rss= -p $(pgrep roea-agent)
   ```

2. **Reduce in-memory cache:**
   ```toml
   [storage]
   page_cache_size = "128MB"
   ```

3. **Restart periodically** (memory leak workaround):
   ```bash
   systemctl --user restart roea-agent
   ```

## Network Issues

### Can't See Network Connections

1. **Check monitor is enabled:**
   ```toml
   [monitor.network]
   enabled = true
   ```

2. **Verify permissions** (may need root for some data)

3. **Check for connections manually:**
   ```bash
   ss -tunap | grep $(pgrep claude)
   ```

### API Endpoints Not Classified

If known APIs show as "Unknown":

1. **Update signatures:**
   ```bash
   roea-cli signatures update
   ```

2. **Add custom endpoint:**
   ```toml
   [[known_endpoints]]
   pattern = "api.myservice.com"
   name = "My Service API"
   ```

## Getting Help

If you can't resolve an issue:

1. **Collect diagnostics:**
   ```bash
   roea-cli diagnose > roea-diagnostics.txt
   ```

2. **Check logs:**
   ```bash
   journalctl -u roea-agent --since "1 hour ago"
   ```

3. **Search issues:**
   [GitHub Issues](https://github.com/your-org/roea-ai/issues)

4. **Open new issue** with diagnostics attached

## See Also

- [FAQ](/reference/faq) - Frequently asked questions
- [Configuration](/reference/configuration) - Configuration reference
- [Contributing](/contributing) - Report bugs
