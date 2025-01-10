use anyhow::{Result, anyhow};
use crate::config::{ProviderConfig, ProviderCredentials};
use super::CloudProvider;
use super::google_drive::GoogleDriveProvider;
use super::onedrive::OneDriveProvider;

/// Cria uma nova instância do provedor baseado na configuração
pub async fn create_provider(config: &ProviderConfig) -> Result<Box<dyn CloudProvider>> {
    match &config.credentials {
        ProviderCredentials::GoogleDrive(creds) => {
            let provider = GoogleDriveProvider::new(
                config.name.clone(),
                creds.client_id.clone(),
                creds.client_secret.clone(),
                creds.token.clone(),
            )?;
            Ok(Box::new(provider))
        }
        ProviderCredentials::OneDrive(creds) => {
            let provider = OneDriveProvider::new(
                config.name.clone(),
                creds.client_id.clone(),
                creds.client_secret.clone(),
                creds.token.clone(),
            )?;
            Ok(Box::new(provider))
        }
    }
} 