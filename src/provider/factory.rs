use anyhow::{anyhow, Result};
use crate::config::ProviderConfig;
use crate::provider::{CloudProvider, google_drive::GoogleDriveProvider, onedrive::OneDriveProvider};

pub async fn create_provider(config: &ProviderConfig) -> Result<Box<dyn CloudProvider>> {
    match &config.credentials.provider_type {
        "googledrive" => {
            let provider = GoogleDriveProvider::new(
                &config.credentials.client_id,
                &config.credentials.client_secret,
            ).await?;
            Ok(Box::new(provider))
        }
        "onedrive" => {
            let provider = OneDriveProvider::new(
                &config.credentials.client_id,
                &config.credentials.client_secret,
            );
            Ok(Box::new(provider))
        }
        _ => Err(anyhow!("Unsupported provider type: {}", config.credentials.provider_type)),
    }
} 