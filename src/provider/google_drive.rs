use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;
use google_drive3::{DriveHub, oauth2, hyper, hyper_rustls};
use tokio::sync::mpsc;
use chrono::{DateTime, Utc};

use super::{CloudProvider, RemoteItem, ChangeType, FolderMapping};

pub struct GoogleDriveProvider {
    #[allow(dead_code)]
    hub: DriveHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    mappings: Vec<FolderMapping>,
}

impl GoogleDriveProvider {
    pub async fn new(
        client_id: String,
        client_secret: String,
        _token_path: Option<String>,
        mappings: Vec<FolderMapping>,
    ) -> Result<Self> {
        let secret = oauth2::ApplicationSecret {
            client_id,
            client_secret,
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            redirect_uris: vec!["urn:ietf:wg:oauth:2.0:oob".to_string()],
            ..Default::default()
        };

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::Interactive,
        ).build().await?;

        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_only()
            .enable_http1()
            .enable_http2()
            .build();

        let hub = DriveHub::new(
            hyper::Client::builder().build(connector),
            auth,
        );

        Ok(Self { hub, mappings })
    }
}

#[async_trait]
impl CloudProvider for GoogleDriveProvider {
    async fn initialize(&mut self) -> Result<()> {
        // Test the connection by trying to list files
        self.list_files("/").await?;
        Ok(())
    }

    async fn list_files(&self, remote_path: &str) -> Result<Vec<RemoteItem>> {
        let (_, file_list) = self.hub
            .files()
            .list()
            .q(&format!("'{}' in parents", remote_path))
            .doit()
            .await?;

        let items = file_list.files.unwrap_or_default()
            .into_iter()
            .map(|file| {
                let modified = file.modified_time
                    .and_then(|t| DateTime::parse_from_rfc3339(&t).ok())
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                RemoteItem {
                    name: file.name.unwrap_or_default(),
                    id: file.id.unwrap_or_default(),
                    size: file.size.unwrap_or_default() as u64,
                    modified,
                    is_folder: file.mime_type.unwrap_or_default() == "application/vnd.google-apps.folder",
                }
            })
            .collect();

        Ok(items)
    }

    async fn upload_file(&self, _local_path: &Path, _remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement file upload
        unimplemented!()
    }

    async fn download_file(&self, _remote_path: &str, _local_path: &Path) -> Result<()> {
        // TODO: Implement file download
        unimplemented!()
    }

    async fn create_directory(&self, _remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement directory creation
        unimplemented!()
    }

    async fn delete(&self, _remote_path: &str) -> Result<()> {
        // TODO: Implement deletion
        unimplemented!()
    }

    async fn exists(&self, _remote_path: &str) -> Result<bool> {
        // TODO: Implement existence check
        unimplemented!()
    }

    async fn get_item(&self, _remote_path: &str) -> Result<Option<RemoteItem>> {
        // TODO: Implement item retrieval
        unimplemented!()
    }

    async fn watch_local_changes(&self, _local_path: &Path, _tx: mpsc::Sender<ChangeType>) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    async fn watch_remote_changes(&self, _remote_path: &str, _tx: mpsc::Sender<RemoteItem>) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    async fn get_mappings(&self) -> Vec<FolderMapping> {
        self.mappings.clone()
    }
} 