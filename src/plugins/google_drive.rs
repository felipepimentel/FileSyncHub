use std::path::Path;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use google_drive::{Client as DriveClient, traits::FileOps};
use bytes::Bytes;
use async_trait::async_trait;
use super::Plugin;
use tokio::time::timeout;
use std::time::Duration;

const AUTH_TIMEOUT_SECS: u64 = 60;

pub struct GoogleDriveClient {
    client: Option<DriveClient>,
    root_folder: String,
}

#[async_trait]
impl Plugin for GoogleDriveClient {
    fn name(&self) -> &str {
        "google_drive"
    }

    async fn upload_chunk(&self, path: &Path, _data: Bytes, _offset: u64) -> anyhow::Result<()> {
        let client = self.client.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Google Drive client not initialized. Call authenticate() first")
        })?;

        let file_name = path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
            .to_string_lossy()
            .to_string();

        let file_metadata = google_drive::types::File {
            name: file_name,
            parents: vec![self.root_folder.clone()],
            ..Default::default()
        };

        client.files().create(
            false,
            "user",
            false,
            "",
            false,
            false,
            false,
            &file_metadata,
        ).await?;

        Ok(())
    }

    async fn download_chunk(&self, path: &str, _offset: u64, _size: usize) -> anyhow::Result<Bytes> {
        let client = self.client.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Google Drive client not initialized. Call authenticate() first")
        })?;

        let response = client.files().download_by_id(path).await?;
        Ok(response.body)
    }

    async fn delete_file(&self, path: &Path) -> anyhow::Result<()> {
        let _client = self.client.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Google Drive client not initialized. Call authenticate() first")
        })?;

        // First, find the file ID by name
        let _file_name = path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
            .to_string_lossy()
            .to_string();

        // TODO: Implement file deletion
        // For now, just return Ok as we haven't implemented the file deletion API yet
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(GoogleDriveClient {
            client: self.client.clone(),
            root_folder: self.root_folder.clone(),
        })
    }
}

impl Clone for GoogleDriveClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            root_folder: self.root_folder.clone(),
        }
    }
}

impl GoogleDriveClient {
    pub async fn new(root_folder: String) -> anyhow::Result<Self> {
        let mut client = Self {
            client: None,
            root_folder,
        };

        match timeout(Duration::from_secs(AUTH_TIMEOUT_SECS), client.authenticate()).await {
            Ok(result) => {
                result?;
                Ok(client)
            }
            Err(_) => {
                Err(anyhow::anyhow!(
                    "Authentication timed out after {} seconds. Please try again.",
                    AUTH_TIMEOUT_SECS
                ))
            }
        }
    }

    async fn authenticate(&mut self) -> anyhow::Result<()> {
        if self.client.is_some() {
            return Ok(());
        }

        let secret = yup_oauth2::read_application_secret("credentials/google_drive.json").await?;
        
        let auth = InstalledFlowAuthenticator::builder(
            secret.clone(),
            InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await?;

        let token_future = auth.token(&["https://www.googleapis.com/auth/drive.file"]);
        let ctrl_c = tokio::signal::ctrl_c();
        
        tokio::select! {
            token_result = token_future => {
                let token = token_result?;
                let client = DriveClient::new(
                    secret.client_id,
                    secret.client_secret,
                    "http://localhost:8080".to_string(),
                    token.token().unwrap_or_default().to_string(),
                    "".to_string(),
                );

                self.client = Some(client);
                Ok(())
            }
            _ = ctrl_c => {
                Err(anyhow::anyhow!("Authentication cancelled by user"))
            }
        }
    }
}
