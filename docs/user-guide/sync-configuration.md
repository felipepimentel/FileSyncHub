# Sync Configuration Guide

This guide explains how to configure file synchronization in FileSyncHub.

## Configuration File

FileSyncHub uses a TOML configuration file located at `~/.config/filesynchub/config.toml`.

### Basic Configuration

```toml
[general]
# Default sync interval in seconds
sync_interval = 300  # 5 minutes

# Log level: "debug", "info", "warn", "error"
log_level = "info"

# Maximum concurrent uploads/downloads
max_concurrent_operations = 4

[sync]
# Local directory to sync
local_dir = "/path/to/your/files"

# Ignore patterns (glob syntax)
ignore_patterns = [
    "*.tmp",
    "*.log",
    ".git/",
    "node_modules/"
]

# File size limits in bytes
max_file_size = 1073741824  # 1GB
min_file_size = 0

[google_drive]
# Root folder for synced files
root_folder = "FileSyncHub"

# Sync specific folders
folders = [
    "Documents",
    "Photos"
]

[onedrive]
# Root folder for synced files
root_folder = "FileSyncHub"

# Sync specific folders
folders = [
    "Work",
    "Personal"
]
```

## Advanced Configuration

### Selective Sync

You can configure which folders to sync for each cloud service:

```toml
[google_drive.sync]
include = [
    "Documents/**/*.pdf",
    "Photos/**/*.{jpg,png}"
]
exclude = [
    "**/*.tmp",
    "**/Thumbs.db"
]

[onedrive.sync]
include = [
    "Work/**/*.docx",
    "Personal/**/*.xlsx"
]
exclude = [
    "**/~$*",
    "**/.DS_Store"
]
```

### Bandwidth Control

Control upload and download speeds:

```toml
[network]
# Bandwidth limits in bytes per second
upload_limit = 1048576    # 1MB/s
download_limit = 2097152  # 2MB/s

# Connection settings
max_retries = 3
retry_delay = 5  # seconds
timeout = 30     # seconds
```

### Conflict Resolution

Configure how file conflicts are handled:

```toml
[sync.conflicts]
# Strategy: "newer", "local", "remote", "rename"
strategy = "newer"

# Backup conflicted files
create_backup = true
backup_dir = "~/.filesynchub/conflicts"

# Notification settings
notify_on_conflict = true
```

### Scheduling

Set up sync schedules:

```toml
[schedule]
# Enable scheduled sync
enabled = true

# Cron-style schedule
schedule = "0 */2 * * *"  # Every 2 hours

# Quiet hours (no sync)
quiet_hours_start = "23:00"
quiet_hours_end = "06:00"
```

## Environment Variables

Override configuration settings with environment variables:

```bash
# General settings
export FILESYNCHUB_SYNC_INTERVAL=600
export FILESYNCHUB_LOG_LEVEL=debug

# Paths
export FILESYNCHUB_LOCAL_DIR="/custom/path"
export FILESYNCHUB_BACKUP_DIR="/backup/path"

# Limits
export FILESYNCHUB_UPLOAD_LIMIT=2097152
export FILESYNCHUB_MAX_FILE_SIZE=2147483648
```

## Command Line Options

Override configuration via command line:

```bash
# Set custom config file
filesynchub --config /path/to/config.toml

# Override sync interval
filesynchub --sync-interval 600

# Set log level
filesynchub --log-level debug

# Specify local directory
filesynchub --local-dir /path/to/sync
```

## Best Practices

1. **File Organization**
   - Keep related files in dedicated folders
   - Use consistent naming conventions
   - Avoid deeply nested directories

2. **Performance**
   - Exclude unnecessary files
   - Set appropriate size limits
   - Configure concurrent operations based on system resources

3. **Network Usage**
   - Set bandwidth limits during work hours
   - Configure quiet hours
   - Use selective sync for large directories

4. **Security**
   - Don't sync sensitive files
   - Use environment variables for credentials
   - Regular backup of configuration

## Troubleshooting

### Common Issues

1. **Sync Not Starting**
   ```bash
   # Check configuration
   filesynchub check-config

   # Verify permissions
   ls -la ~/.config/filesynchub/
   ```

2. **High Resource Usage**
   ```toml
   [general]
   # Reduce concurrent operations
   max_concurrent_operations = 2

   # Increase sync interval
   sync_interval = 600
   ```

3. **Network Problems**
   ```toml
   [network]
   # Increase timeouts
   timeout = 60
   retry_delay = 10
   max_retries = 5
   ```

## Examples

### Basic Sync Setup
```toml
[general]
sync_interval = 300
log_level = "info"

[sync]
local_dir = "~/Documents"
ignore_patterns = ["*.tmp"]

[google_drive]
root_folder = "Backup"
```

### Advanced Multi-Service Setup
```toml
[sync]
local_dir = "~/Projects"
ignore_patterns = [
    "*.tmp",
    "node_modules/",
    "target/",
    ".git/"
]

[google_drive]
root_folder = "Work"
folders = ["Documentation", "Resources"]

[google_drive.sync]
include = ["**/*.{md,pdf,docx}"]
exclude = ["**/draft/"]

[onedrive]
root_folder = "Projects"
folders = ["Code", "Assets"]

[onedrive.sync]
include = ["**/*.{rs,toml,json}"]
exclude = ["**/target/"]
```

## Next Steps

- [Advanced Features](advanced-features.md)
- [Automated Syncing](automated-sync.md)
- [Troubleshooting Guide](troubleshooting.md) 