---
layout: default
title: User Guide
nav_order: 2
has_children: true
---

# FileSyncHub User Guide

Welcome to the FileSyncHub User Guide! This comprehensive guide will help you get started with FileSyncHub and make the most of its features.

## Getting Started

### Installation

1. Install using Cargo:
   ```bash
   cargo install filesynchub
   ```

2. Or build from source:
   ```bash
   git clone https://github.com/yourusername/FileSyncHub.git
   cd FileSyncHub
   cargo build --release
   ```

### Initial Setup

1. Create the configuration directory:
   ```bash
   mkdir -p ~/.config/filesynchub/credentials
   ```

2. Configure your cloud services:
   - [Google Drive Setup](google-drive-setup.md)
   - [OneDrive Setup](onedrive-setup.md)

## Basic Usage

### Starting FileSyncHub

Run FileSyncHub with the TUI interface:
```bash
filesynchub sync --tui
```

### First-Time Authentication

1. When you first run FileSyncHub, it will prompt you to authenticate with your cloud services
2. Follow the authentication instructions in the TUI
3. Complete the authentication process in your web browser
4. Return to FileSyncHub to begin syncing

## Configuration

### Basic Configuration

Create a configuration file at `~/.config/filesynchub/config.toml`:

```toml
[general]
sync_interval = 300  # 5 minutes
log_level = "info"

[sync]
local_dir = "/path/to/your/files"
```

See [Sync Configuration](sync-configuration.md) for detailed configuration options.

## Features

### File Synchronization

- Bi-directional sync between local files and cloud storage
- Real-time file change detection
- Conflict resolution
- Selective sync with ignore patterns

### Cloud Services

- Google Drive integration
- OneDrive integration
- Extensible plugin system for additional services

### User Interface

- Beautiful Terminal User Interface (TUI)
- Progress indicators
- File transfer status
- Error reporting

## Advanced Topics

- [Advanced Configuration](advanced-configuration.md)
- [Plugin Development](plugin-development.md)
- [Automated Syncing](automated-sync.md)
- [Troubleshooting](troubleshooting.md)

## Best Practices

### Security

1. Keep credentials secure
2. Use environment variables for sensitive information
3. Regularly update FileSyncHub
4. Review sync permissions

### Performance

1. Configure appropriate sync intervals
2. Use selective sync for large directories
3. Set reasonable file size limits
4. Optimize network settings

### Organization

1. Structure your sync directories logically
2. Use consistent naming conventions
3. Configure ignore patterns for unnecessary files
4. Keep sync paths shallow

## Common Tasks

### Adding Files to Sync

1. Place files in your configured sync directory
2. FileSyncHub will automatically detect and sync them
3. Monitor progress in the TUI

### Resolving Conflicts

1. FileSyncHub detects file conflicts
2. Choose resolution strategy in configuration
3. Review conflict logs
4. Manually resolve if needed

### Checking Sync Status

1. Open FileSyncHub TUI
2. View current sync status
3. Check logs for detailed information
4. Monitor file transfer progress

## Troubleshooting

### Common Issues

1. Authentication Problems
   - Verify credentials
   - Check network connection
   - Review authentication logs

2. Sync Issues
   - Check file permissions
   - Verify configuration
   - Review ignore patterns

3. Performance Problems
   - Adjust sync interval
   - Optimize file patterns
   - Check resource usage

## Next Steps

- Read the [Configuration Guide](sync-configuration.md)
- Set up [Cloud Services](cloud-services.md)
- Explore [Advanced Features](advanced-features.md)
- Join our [Community](../community.md)

## Additional Resources

- [API Documentation](../api/index.md)
- [Contributing Guide](../contributing/index.md)
- [GitHub Repository](https://github.com/yourusername/FileSyncHub)
- [Issue Tracker](https://github.com/yourusername/FileSyncHub/issues) 