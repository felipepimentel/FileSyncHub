use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
pub struct ProviderCredentials {
    #[serde(rename = "type")]
    pub provider_type: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMapping {
    pub local_path: PathBuf,
    pub remote_path: String,
}

impl Config {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
