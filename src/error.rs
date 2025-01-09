use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSyncError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Synchronization error: {0}")]
    Sync(String),

    #[error("Safety error: {0}")]
    Safety(String),

    #[error("Performance error: {0}")]
    Performance(String),

    #[error("Logging error: {0}")]
    Log(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("File corrupted: {0}")]
    FileCorrupted(PathBuf),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Operation cancelled: {0}")]
    Cancelled(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<toml::de::Error> for FileSyncError {
    fn from(err: toml::de::Error) -> Self {
        FileSyncError::Config(format!("Error reading configuration: {}", err))
    }
}

impl From<toml::ser::Error> for FileSyncError {
    fn from(err: toml::ser::Error) -> Self {
        FileSyncError::Config(format!("Error saving configuration: {}", err))
    }
}

impl From<serde_json::Error> for FileSyncError {
    fn from(err: serde_json::Error) -> Self {
        FileSyncError::Serialization(format!("JSON error: {}", err))
    }
}

impl From<reqwest::Error> for FileSyncError {
    fn from(err: reqwest::Error) -> Self {
        FileSyncError::Network(format!("HTTP request error: {}", err))
    }
}

impl From<oauth2::basic::BasicRequestTokenError> for FileSyncError {
    fn from(err: oauth2::basic::BasicRequestTokenError) -> Self {
        FileSyncError::Auth(format!("OAuth2 authentication error: {}", err))
    }
}

impl From<notify::Error> for FileSyncError {
    fn from(err: notify::Error) -> Self {
        FileSyncError::Io(io::Error::new(io::ErrorKind::Other, err))
    }
}

impl From<log::SetLoggerError> for FileSyncError {
    fn from(err: log::SetLoggerError) -> Self {
        FileSyncError::Log(format!("Error configuring logger: {}", err))
    }
}

impl From<crossterm::ErrorKind> for FileSyncError {
    fn from(err: crossterm::ErrorKind) -> Self {
        FileSyncError::Tui(format!("Terminal UI error: {}", err))
    }
}

impl From<daemonize::DaemonizeError> for FileSyncError {
    fn from(err: daemonize::DaemonizeError) -> Self {
        FileSyncError::Service(format!("Service error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, FileSyncError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        // Test IO error conversion
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let sync_err: FileSyncError = io_err.into();
        assert!(matches!(sync_err, FileSyncError::Io(_)));

        // Test configuration error
        let config_err = FileSyncError::Config("invalid configuration".to_string());
        assert!(matches!(config_err, FileSyncError::Config(_)));

        // Test authentication error
        let auth_err = FileSyncError::Auth("invalid token".to_string());
        assert!(matches!(auth_err, FileSyncError::Auth(_)));

        // Test plugin error
        let plugin_err = FileSyncError::PluginNotFound("google_drive".to_string());
        assert!(matches!(plugin_err, FileSyncError::PluginNotFound(_)));

        // Test service error
        let service_err = FileSyncError::Service("failed to start".to_string());
        assert!(matches!(service_err, FileSyncError::Service(_)));
    }

    #[test]
    fn test_error_display() {
        let err = FileSyncError::FileNotFound(PathBuf::from("test.txt"));
        assert_eq!(err.to_string(), "File not found: test.txt");

        let err = FileSyncError::Network("connection lost".to_string());
        assert_eq!(err.to_string(), "Network error: connection lost");

        let err = FileSyncError::RateLimit("too many requests".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: too many requests");

        let err = FileSyncError::QuotaExceeded("storage limit reached".to_string());
        assert_eq!(err.to_string(), "Quota exceeded: storage limit reached");
    }
}
