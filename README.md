# FileSyncHub

A file synchronization hub with support for Google Drive and OneDrive, written in Rust.

## Features

- Real-time file synchronization
- Support for multiple cloud storage providers
- Terminal user interface (TUI) for monitoring sync progress
- Background service mode
- Configurable file patterns and sync rules
- Efficient chunked file uploads
- Secure OAuth2 authentication

## Installation

1. Clone the repository:

```bash
git clone https://github.com/felipepimentel/FileSyncHub.git
cd FileSyncHub
```

2. Build the project:

```bash
cargo build --release
```

## Configuration

1. Copy the example configuration file:

```bash
cp examples/config/example_config.toml config.toml
```

2. Edit `config.toml` to set your sync directories and preferences.

3. Set up cloud provider credentials:

### Google Drive

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Enable the Google Drive API
4. Create OAuth 2.0 credentials
5. Copy the example credentials file:

```bash
cp examples/credentials/google_drive_example.json credentials/google_drive.json
```

6. Update the file with your credentials

### OneDrive

1. Go to the [Microsoft Azure Portal](https://portal.azure.com/)
2. Register a new application
3. Add Microsoft Graph API permissions for Files.ReadWrite.All
4. Copy the example credentials file:

```bash
cp examples/credentials/onedrive_example.json credentials/onedrive.json
```

5. Update the file with your credentials

## Running Examples

The project includes several examples demonstrating different features:

### 1. Basic Synchronization

Test basic file operations with Google Drive:

```bash
cargo run --example sync_with_google_drive
```

Test basic file operations with OneDrive:

```bash
cargo run --example sync_with_onedrive
```

### 2. Document Synchronization

Sync document files between local storage and cloud providers:

```bash
cargo run --example sync_documents
```

### 3. TUI Mode

Run with the terminal user interface to monitor sync progress:

```bash
cargo run --example sync_with_tui
```

### 4. Service Mode

Run as a background service:

```bash
cargo run --example run_as_service
```

## Usage

### Running as a Service

Start FileSyncHub as a background service:

```bash
filesynchub service start
```

Stop the service:

```bash
filesynchub service stop
```

### Running with TUI

Start FileSyncHub with the terminal user interface:

```bash
filesynchub tui
```

### Command Line Options

```bash
filesynchub [OPTIONS] [COMMAND]

Commands:
  service    Run as a background service
  tui        Run with terminal user interface
  help       Print help information

Options:
  -c, --config <FILE>    Configuration file path
  -v, --verbose          Enable verbose logging
  -h, --help            Print help information
  -V, --version         Print version information
```

## Development

### Running Tests

Run the test suite:

```bash
cargo test
```

Run tests with logging:

```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Building Documentation

Generate and view the documentation:

```bash
cargo doc --no-deps --open
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
