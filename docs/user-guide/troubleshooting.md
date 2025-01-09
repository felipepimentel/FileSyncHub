---
layout: default
title: Troubleshooting
parent: User Guide
nav_order: 4
---

# Troubleshooting Guide

This guide helps you diagnose and resolve common issues you might encounter while using FileSyncHub.

## Authentication Issues

### Google Drive Authentication Fails

**Symptoms:**
- "Invalid client credentials" error
- Authentication popup doesn't appear
- Token refresh fails

**Solutions:**

1. Verify client credentials:
   ```bash
   # Check if credentials are correctly set
   cat ~/.config/filesynchub/google_drive_credentials.json
   ```

2. Reset authentication:
   ```bash
   # Remove existing tokens
   rm ~/.config/filesynchub/google_drive_token.json
   # Restart FileSyncHub to trigger new authentication
   ```

3. Check OAuth configuration:
   - Ensure redirect URI matches in Google Cloud Console
   - Verify application is not in testing mode
   - Check if required scopes are enabled

### OneDrive Authentication Issues

**Symptoms:**
- "Invalid client ID" error
- Authentication timeout
- Permissions errors

**Solutions:**

1. Verify Azure AD configuration:
   - Check client ID and secret
   - Ensure redirect URI is registered
   - Verify required permissions are granted

2. Clear token cache:
   ```bash
   rm ~/.config/filesynchub/onedrive_token.json
   ```

## Sync Issues

### Files Not Syncing

**Symptoms:**
- Files appear unchanged
- Sync seems stuck
- No progress indication

**Solutions:**

1. Check file permissions:
   ```bash
   # Verify file permissions
   ls -la ~/Documents/sync
   
   # Fix permissions if needed
   chmod -R u+rw ~/Documents/sync
   ```

2. Verify sync configuration:
   ```bash
   # Check sync settings
   cat ~/.config/filesynchub/config.toml
   
   # Ensure sync directory is correct
   echo $FILESYNCHUB_SYNC_DIR
   ```

3. Check ignore patterns:
   ```toml
   # config.toml
   [sync]
   ignore_patterns = [
       "*.tmp",
       ".git/"
   ]
   ```

### Sync Conflicts

**Symptoms:**
- Duplicate files
- `.conflict` files appearing
- Sync errors

**Solutions:**

1. Review conflict resolution settings:
   ```toml
   [sync]
   conflict_resolution = "newer"  # or "keep_both", "ask"
   ```

2. Check conflict logs:
   ```bash
   # View conflict logs
   cat ~/.config/filesynchub/logs/conflicts.log
   ```

3. Manually resolve conflicts:
   ```bash
   # List conflict files
   find ~/Documents/sync -name "*.conflict"
   
   # Review and resolve each conflict
   mv file.conflict file_resolved
   ```

## Performance Issues

### Slow Sync Speed

**Symptoms:**
- High CPU usage
- Slow file transfers
- Long sync times

**Solutions:**

1. Adjust chunk size:
   ```toml
   [performance]
   chunk_size = 16777216  # 16MB
   concurrent_chunks = 4
   ```

2. Check network settings:
   ```toml
   [network]
   max_concurrent_transfers = 4
   timeout = 30
   ```

3. Monitor resource usage:
   ```bash
   # Check CPU and memory usage
   top -p $(pgrep filesynchub)
   
   # Monitor network usage
   iftop -P
   ```

### High Memory Usage

**Symptoms:**
- System slowdown
- Out of memory errors
- Swap usage increases

**Solutions:**

1. Adjust memory limits:
   ```toml
   [resources]
   max_memory = "1GB"
   buffer_size = "32MB"
   cache_size = "256MB"
   ```

2. Monitor memory usage:
   ```bash
   # Check memory usage
   ps -o pid,user,%mem,command ax | grep filesynchub
   ```

## Network Issues

### Connection Problems

**Symptoms:**
- Timeout errors
- Connection refused
- Network unreachable

**Solutions:**

1. Check proxy settings:
   ```toml
   [network]
   proxy_url = "http://proxy.example.com:8080"
   proxy_username = "user"
   proxy_password = "pass"
   ```

2. Verify network connectivity:
   ```bash
   # Test connectivity
   ping 8.8.8.8
   
   # Check DNS resolution
   nslookup drive.google.com
   ```

3. Adjust timeouts:
   ```toml
   [network]
   timeout = 60  # seconds
   retry_count = 5
   retry_delay = 10
   ```

### SSL/TLS Issues

**Symptoms:**
- Certificate validation errors
- SSL handshake failed
- Insecure connection warnings

**Solutions:**

1. Update SSL settings:
   ```toml
   [security]
   verify_ssl = true
   minimum_tls_version = "1.2"
   ```

2. Check system certificates:
   ```bash
   # Update system certificates
   sudo update-ca-certificates
   ```

## Plugin Issues

### Plugin Loading Fails

**Symptoms:**
- Plugin not found
- Initialization errors
- Missing dependencies

**Solutions:**

1. Verify plugin installation:
   ```bash
   # Check plugin directory
   ls ~/.config/filesynchub/plugins/
   ```

2. Check plugin configuration:
   ```toml
   [plugins]
   plugin_dir = "~/.config/filesynchub/plugins"
   
   [plugins.google_drive]
   enabled = true
   ```

3. Review plugin logs:
   ```bash
   # Check plugin-specific logs
   cat ~/.config/filesynchub/logs/plugins.log
   ```

## Logging and Debugging

### Enable Debug Logging

To get more detailed logs:

```toml
[logging]
level = "debug"
format = "json"

[logging.file]
path = "/var/log/filesynchub/sync.log"
max_size = "100MB"
max_files = 5
```

### Common Debug Commands

```bash
# View real-time logs
tail -f ~/.config/filesynchub/logs/sync.log

# Search for errors
grep "ERROR" ~/.config/filesynchub/logs/sync.log

# Check system journal
journalctl -u filesynchub
```

## Common Error Messages

### "Invalid Configuration"

**Cause:** Configuration file syntax error or invalid values

**Solution:**
1. Validate TOML syntax
2. Check configuration values
3. Use default configuration as reference

### "Permission Denied"

**Cause:** Insufficient file or directory permissions

**Solution:**
1. Check file ownership
2. Verify directory permissions
3. Adjust umask settings

### "Resource Temporarily Unavailable"

**Cause:** System resource limits reached

**Solution:**
1. Check system limits
2. Adjust resource configuration
3. Monitor system usage

## Getting Help

If you're still experiencing issues:

1. Check the [GitHub Issues](https://github.com/felipepimentel/filesynchub/issues)
2. Search the [Documentation](https://felipepimentel.github.io/filesynchub/)
3. Create a new issue with:
   - Error messages
   - Configuration files
   - Log outputs
   - Steps to reproduce 