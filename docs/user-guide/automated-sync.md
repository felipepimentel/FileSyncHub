---
layout: default
title: Automated Sync
parent: User Guide
nav_order: 7
---

# Automated Sync Guide

This guide explains how to set up automated file synchronization with FileSyncHub.

## Systemd Service Setup

### Create Service File

Create a systemd service file at `/etc/systemd/system/filesynchub.service`:

```ini
[Unit]
Description=FileSyncHub File Synchronization Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=your_username
ExecStart=/usr/local/bin/filesynchub sync --daemon
Restart=always
RestartSec=10

# Environment variables
Environment=FILESYNCHUB_CONFIG_DIR=/home/your_username/.config/filesynchub
Environment=RUST_LOG=info

# Security
NoNewPrivileges=true
ProtectSystem=full
ProtectHome=read-only
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

### Enable and Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service
sudo systemctl enable filesynchub

# Start service
sudo systemctl start filesynchub

# Check status
sudo systemctl status filesynchub
```

## Cron Job Setup

### Basic Cron Setup

Add to crontab (`crontab -e`):

```bash
# Run every 5 minutes
*/5 * * * * /usr/local/bin/filesynchub sync --quiet

# Run during work hours only (9 AM to 5 PM, Monday to Friday)
*/10 9-17 * * 1-5 /usr/local/bin/filesynchub sync --quiet
```

### Advanced Cron Configuration

Create a script at `~/.local/bin/filesynchub-sync.sh`:

```bash
#!/bin/bash

# Environment setup
export FILESYNCHUB_CONFIG_DIR="$HOME/.config/filesynchub"
export RUST_LOG=info

# Check internet connection
if ! ping -c 1 8.8.8.8 &> /dev/null; then
    echo "No internet connection"
    exit 1
fi

# Run sync with logging
/usr/local/bin/filesynchub sync --quiet \
    >> "$HOME/.local/share/filesynchub/sync.log" 2>&1
```

Add to crontab:

```bash
# Run script every 15 minutes
*/15 * * * * $HOME/.local/bin/filesynchub-sync.sh
```

## Automated Startup

### Desktop Autostart

Create `~/.config/autostart/filesynchub.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=FileSyncHub
Comment=File Synchronization Service
Exec=filesynchub sync --tui
Terminal=false
Categories=Utility;
```

### Command Line Autostart

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# Start FileSyncHub in background if not running
if ! pgrep -x "filesynchub" > /dev/null; then
    filesynchub sync --daemon &
fi
```

## Schedule Configuration

### Time-Based Scheduling

Configure in `~/.config/filesynchub/config.toml`:

```toml
[schedule]
# Enable scheduling
enabled = true

# Cron-style schedule
schedule = "*/15 * * * *"  # Every 15 minutes

# Quiet hours
quiet_hours_start = "23:00"
quiet_hours_end = "06:00"

# Day restrictions
working_days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
weekend_schedule = "0 */4 * * 6,0"  # Every 4 hours on weekends
```

### Event-Based Triggers

```toml
[triggers]
# Network events
on_network_connect = true
on_vpn_connect = false

# Power events
on_ac_power = true
on_battery = false
battery_threshold = 20  # Minimum battery percentage

# System events
on_system_idle = true
idle_timeout = 300  # Seconds
on_screen_lock = false
```

## Resource Management

### CPU and Memory Limits

```toml
[resources]
# CPU usage
max_cpu_percent = 50
nice_level = 10

# Memory limits
max_memory_mb = 512
swap_allowed = false

# I/O limits
io_priority = "idle"
io_class = "best-effort"
```

### Network Bandwidth

```toml
[network]
# Global rate limits
upload_limit = "1MB/s"
download_limit = "2MB/s"

# Time-based limits
[network.schedule]
peak_hours = "9-17"
peak_upload_limit = "512KB/s"
peak_download_limit = "1MB/s"
off_peak_upload_limit = "5MB/s"
off_peak_download_limit = "10MB/s"
```

## Monitoring and Logging

### Log Configuration

```toml
[logging]
# File logging
log_file = "/var/log/filesynchub/sync.log"
log_level = "info"
max_log_size = "10MB"
max_log_files = 5

# Syslog integration
use_syslog = true
syslog_identifier = "filesynchub"
```

### Status Monitoring

```toml
[monitoring]
# Status file
status_file = "/var/run/filesynchub/status.json"
status_update_interval = 60

# Health checks
health_check_url = "http://localhost:8080/health"
health_check_interval = 300
```

## Notifications

### Desktop Notifications

```toml
[notifications]
# Enable notifications
enable = true
level = "error"  # or "info", "warning"

# Desktop notifications
desktop_notifications = true
notification_timeout = 5000  # milliseconds

# Sound alerts
sound_alerts = false
sound_file = "/usr/share/sounds/freedesktop/stereo/complete.oga"
```

### Email Notifications

```toml
[notifications.email]
enabled = true
smtp_server = "smtp.gmail.com"
smtp_port = 587
smtp_username = "your.email@gmail.com"
smtp_password = "${SMTP_PASSWORD}"  # Use environment variable
recipient = "admin@example.com"
```

## Recovery and Maintenance

### Auto-Recovery

```toml
[recovery]
# Auto-restart on failure
max_retries = 3
retry_delay = 60  # seconds

# Health checks
health_check_enabled = true
health_check_interval = 300
auto_repair = true

# State management
state_backup_interval = 3600
max_state_backups = 5
```

### Maintenance Schedule

```toml
[maintenance]
# Database cleanup
db_vacuum_schedule = "0 0 * * 0"  # Weekly
db_backup_schedule = "0 0 * * *"  # Daily

# Cache management
cache_cleanup_schedule = "0 */6 * * *"  # Every 6 hours
max_cache_size = "1GB"
min_free_space = "10GB"
```

## Example Configurations

### Basic Automated Setup

```toml
[schedule]
enabled = true
schedule = "*/15 * * * *"

[resources]
max_cpu_percent = 30
max_memory_mb = 256

[network]
upload_limit = "1MB/s"
download_limit = "2MB/s"

[logging]
log_file = "~/.local/share/filesynchub/sync.log"
log_level = "info"
```

### Enterprise Setup

```toml
[schedule]
enabled = true
schedule = "*/5 * * * *"
working_days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
quiet_hours_start = "18:00"
quiet_hours_end = "06:00"

[resources]
max_cpu_percent = 50
max_memory_mb = 1024
io_priority = "best-effort"

[network]
upload_limit = "5MB/s"
download_limit = "10MB/s"

[network.schedule]
peak_hours = "9-17"
peak_upload_limit = "2MB/s"
peak_download_limit = "5MB/s"

[monitoring]
status_file = "/var/run/filesynchub/status.json"
health_check_enabled = true

[notifications.email]
enabled = true
smtp_server = "smtp.company.com"
recipient = "it-support@company.com"

[maintenance]
db_backup_schedule = "0 0 * * *"
cache_cleanup_schedule = "0 */4 * * *"
``` 