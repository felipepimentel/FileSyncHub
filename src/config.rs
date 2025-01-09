use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub root_dir: PathBuf,
    pub plugins: HashMap<String, PluginConfig>,
    pub sync: SyncConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub root_folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub interval: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            root_dir: PathBuf::new(),
            plugins: HashMap::new(),
            sync: SyncConfig {
                interval: 300,
                max_retries: 3,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
            },
        }
    }

    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        Ok(config)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;
        std::fs::write(path, content).with_context(|| format!("Failed to write config file: {}", path))?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if !self.root_dir.exists() {
            anyhow::bail!("Root directory does not exist: {:?}", self.root_dir);
        }

        if !self.root_dir.is_dir() {
            anyhow::bail!("Root path is not a directory: {:?}", self.root_dir);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.toml");

        // Create a test configuration
        let mut config = Config::new();
        config.root_dir = temp_dir.path().join("sync");

        // Add a test plugin configuration
        let plugin_config = PluginConfig {
            enabled: true,
            root_folder: "test".to_string(),
        };
        config
            .plugins
            .insert("test_plugin".to_string(), plugin_config);

        // Save the configuration
        config.save(config_path.to_str().unwrap())?;

        // Load and validate the configuration
        let loaded_config = Config::load(config_path.to_str().unwrap())?;
        assert_eq!(loaded_config.root_dir, config.root_dir);
        assert_eq!(
            loaded_config
                .plugins
                .get("test_plugin")
                .unwrap()
                .root_folder,
            "test"
        );

        Ok(())
    }
}
