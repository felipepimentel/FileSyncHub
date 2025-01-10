use anyhow::Result;

use crate::config::{ProviderConfig, ProviderCredentials};
use super::{CloudProvider, google_drive::GoogleDriveProvider, onedrive::OneDriveProvider};

pub async fn create_provider(config: &ProviderConfig) -> Result<Box<dyn CloudProvider>> {
    match &config.credentials {
        ProviderCredentials::GoogleDrive(creds) => {
            let provider = GoogleDriveProvider::new(
                creds.client_id.clone(),
                creds.client_secret.clone(),
                Some(format!("token_{}.json", config.name)),
            ).await?;
            Ok(Box::new(provider))
        }
        ProviderCredentials::OneDrive(creds) => {
            let provider = OneDriveProvider::new(
                creds.client_id.clone(),
                creds.client_secret.clone(),
            );
            Ok(Box::new(provider))
        }
    }
} 