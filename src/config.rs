use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub credentials: ProviderCredentials,
    pub mappings: Vec<FolderMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderCredentials {
    #[serde(rename = "googledrive")]
    GoogleDrive(GoogleDriveCredentials),
    #[serde(rename = "onedrive")]
    OneDrive(OneDriveCredentials),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDriveCredentials {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveCredentials {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMapping {
    pub local_path: PathBuf,
    pub remote_path: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub async fn from_file(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub async fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}
