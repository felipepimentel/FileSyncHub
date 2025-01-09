---
layout: default
title: Advanced Configuration
parent: User Guide
nav_order: 3
---

# Advanced Configuration Guide

This guide covers advanced configuration options for FileSyncHub, including detailed TOML configuration, resource management, monitoring, and logging.

## TOML Configuration

FileSyncHub uses TOML for its configuration file. Here's a comprehensive example with all available options:

```toml
# General configuration
[general]
sync_dir = "~/Documents/sync"
log_level = "debug"
max_concurrent_tasks = 4

# Sync configuration
[sync]
interval = 300  # seconds
ignore_patterns = [
    "*.tmp",
    "*.log",
    ".git/",
    "node_modules/"
]
max_file_size = 104857600  # 100MB in bytes
delete_mode = "trash"  # Options: "trash", "permanent"

# Network configuration
[network]
max_concurrent_transfers = 4
timeout = 30  # seconds
retry_count = 3
retry_delay = 5  # seconds
proxy_url = "http://proxy.example.com:8080"  # optional

# Google Drive plugin configuration
[plugins.google_drive]
enabled = true
root_folder = "FileSyncHub"
chunk_size = 16777216  # 16MB in bytes
client_id = "your-client-id"
client_secret = "your-client-secret"
token_path = "~/.config/filesynchub/google_drive_token.json"

# OneDrive plugin configuration
[plugins.onedrive]
enabled = true
root_folder = "FileSyncHub"
chunk_size = 16777216  # 16MB in bytes
client_id = "your-client-id"
client_secret = "your-client-secret"
token_path = "~/.config/filesynchub/onedrive_token.json"

# Monitoring configuration
[monitoring]
enabled = true
metrics_port = 9090
health_check_port = 8080
prometheus_enabled = true

# Notification configuration
[notifications]
enabled = true
mode = ["email", "desktop"]

[notifications.email]
smtp_server = "smtp.gmail.com"
smtp_port = 587
username = "your-email@gmail.com"
password = "your-app-password"
recipients = ["notify@example.com"]

[notifications.desktop]
sound = true
icon = true
```

## Resource Management

### Memory Management

Control memory usage with these settings:

```toml
[resources]
max_memory = "2GB"
buffer_size = "64MB"
cache_size = "512MB"
```

### CPU Usage

Manage CPU utilization:

```toml
[resources.cpu]
max_threads = 4
priority = "normal"  # Options: "low", "normal", "high"
```

### Disk Usage

Configure disk-related settings:

```toml
[resources.disk]
min_free_space = "10GB"
temp_dir = "/tmp/filesynchub"
cleanup_interval = 3600  # seconds
```

## Monitoring and Logging

### Prometheus Metrics

Enable Prometheus metrics:

```toml
[monitoring.prometheus]
enabled = true
port = 9090
path = "/metrics"
labels = { environment = "production", region = "us-west" }
```

### Logging Configuration

Detailed logging settings:

```toml
[logging]
level = "debug"
format = "json"
output = ["file", "console"]

[logging.file]
path = "/var/log/filesynchub/sync.log"
max_size = "100MB"
max_files = 5
compress = true

[logging.console]
colored = true
timestamp = true
```

### Health Checks

Configure health monitoring:

```toml
[monitoring.health]
enabled = true
port = 8080
path = "/health"
interval = 60  # seconds
timeout = 5  # seconds
```

## Advanced Plugin Configuration

### Custom Plugin Settings

Example of custom plugin configuration:

```toml
[plugins.custom]
enabled = true
type = "s3"
endpoint = "https://s3.amazonaws.com"
bucket = "my-sync-bucket"
region = "us-west-2"
access_key = "your-access-key"
secret_key = "your-secret-key"
```

### Plugin Priorities

Set sync priorities for different plugins:

```toml
[plugins.priorities]
google_drive = 1
onedrive = 2
custom = 3
```

## Security Configuration

### Encryption Settings

Configure file encryption:

```toml
[security.encryption]
enabled = true
algorithm = "AES-256-GCM"
key_file = "~/.config/filesynchub/encryption.key"
```

### Authentication

Configure authentication methods:

```toml
[security.auth]
method = "oauth2"
token_refresh_window = 3600  # seconds
max_token_age = 2592000  # 30 days in seconds
```

## Performance Tuning

### Transfer Optimization

Configure transfer settings:

```toml
[performance.transfer]
chunk_size = 16777216  # 16MB
concurrent_chunks = 4
compression = true
compression_level = 6
```

### Caching

Configure caching behavior:

```toml
[performance.cache]
enabled = true
size = "1GB"
ttl = 3600  # seconds
cleanup_interval = 300  # seconds
```

## Example Configurations

### Basic Setup

Minimal configuration for simple use:

```toml
[general]
sync_dir = "~/Documents/sync"

[plugins.google_drive]
enabled = true
client_id = "your-client-id"
client_secret = "your-client-secret"
```

### Enterprise Setup

Full-featured configuration for enterprise use:

```toml
[general]
sync_dir = "/data/sync"
log_level = "info"
max_concurrent_tasks = 8

[sync]
interval = 300
ignore_patterns = ["*.tmp", "*.log", ".git/"]
max_file_size = 1073741824  # 1GB

[network]
max_concurrent_transfers = 8
timeout = 60
retry_count = 5

[plugins.google_drive]
enabled = true
root_folder = "Enterprise"
chunk_size = 33554432  # 32MB
client_id = "your-client-id"
client_secret = "your-client-secret"

[monitoring]
enabled = true
metrics_port = 9090
prometheus_enabled = true

[security.encryption]
enabled = true
algorithm = "AES-256-GCM"

[performance]
chunk_size = 33554432  # 32MB
concurrent_chunks = 8
compression = true
```

## Environment Variables

All configuration options can be overridden using environment variables:

```bash
export FILESYNCHUB_SYNC_DIR="/custom/sync/dir"
export FILESYNCHUB_LOG_LEVEL="debug"
export FILESYNCHUB_GOOGLE_DRIVE_CLIENT_ID="your-client-id"
export FILESYNCHUB_GOOGLE_DRIVE_CLIENT_SECRET="your-client-secret"
```

The environment variable format is:
- Uppercase
- Prefixed with `FILESYNCHUB_`
- Nested config separated by underscores
- Arrays use numeric indices 