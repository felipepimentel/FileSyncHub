use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;
use google_drive3::{DriveHub, oauth2, hyper, hyper_rustls};
use tokio::sync::mpsc;

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
    async fn list_files(&self, _remote_path: &str) -> Result<Vec<RemoteItem>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn upload_file(&self, _local_path: &Path, _remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement
        unimplemented!()
    }

    async fn download_file(&self, _remote_path: &str, _local_path: &Path) -> Result<()> {
        // TODO: Implement
        unimplemented!()
    }

    async fn create_directory(&self, _remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement
        unimplemented!()
    }

    async fn delete(&self, _remote_path: &str) -> Result<()> {
        // TODO: Implement
        unimplemented!()
    }

    async fn exists(&self, _remote_path: &str) -> Result<bool> {
        // TODO: Implement
        Ok(false)
    }

    async fn get_item(&self, _remote_path: &str) -> Result<Option<RemoteItem>> {
        // TODO: Implement
        Ok(None)
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