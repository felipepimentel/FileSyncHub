use anyhow::{Result, Context};
use async_trait::async_trait;
use google_drive3::{DriveHub, oauth2, hyper, hyper_rustls};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use notify::{Watcher, RecursiveMode};
use std::sync::Arc;
use tokio::sync::Mutex;
use super::{CloudProvider, RemoteItem, ChangeType};

pub struct GoogleDriveProvider {
    name: String,
    hub: Arc<Mutex<DriveHub>>,
    auth: oauth2::authenticator::Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl GoogleDriveProvider {
    pub fn new(
        name: String,
        client_id: String,
        client_secret: String,
        token: Option<String>,
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
            oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk(format!("{}_token.json", name))
        .build()
        .context("Failed to create authenticator")?;

        let hub = DriveHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            auth.clone(),
        );

        Ok(Self {
            name,
            hub: Arc::new(Mutex::new(hub)),
            auth,
        })
    }

    async fn get_file_id(&self, path: &str) -> Result<Option<String>> {
        let query = format!(
            "name = '{}' and trashed = false",
            Path::new(path).file_name().unwrap().to_str().unwrap()
        );

        let hub = self.hub.lock().await;
        let result = hub
            .files()
            .list()
            .q(&query)
            .spaces("drive")
            .fields("files(id, name)")
            .doit()
            .await
            .context("Failed to list files")?;

        Ok(result.1.files.and_then(|files| files.first().map(|f| f.id.clone())))
    }
}

#[async_trait]
impl CloudProvider for GoogleDriveProvider {
    async fn initialize(&mut self) -> Result<()> {
        // A autenticação já é feita no new()
        Ok(())
    }

    async fn list_files(&self, remote_path: &str) -> Result<Vec<RemoteItem>> {
        let hub = self.hub.lock().await;
        let query = format!("'{}' in parents and trashed = false", remote_path);

        let result = hub
            .files()
            .list()
            .q(&query)
            .spaces("drive")
            .fields("files(id, name, mimeType, modifiedTime, size)")
            .doit()
            .await
            .context("Failed to list files")?;

        let files = result.1.files.unwrap_or_default();
        let mut items = Vec::new();

        for file in files {
            items.push(RemoteItem {
                id: file.id.unwrap_or_default(),
                name: file.name.unwrap_or_default(),
                path: format!("{}/{}", remote_path, file.name.unwrap_or_default()),
                is_dir: file.mime_type.unwrap_or_default() == "application/vnd.google-apps.folder",
                modified_at: chrono::DateTime::parse_from_rfc3339(&file.modified_time.unwrap_or_default())
                    .unwrap_or_default()
                    .into(),
                size: file.size.unwrap_or_default().parse().unwrap_or_default(),
            });
        }

        Ok(items)
    }

    async fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<RemoteItem> {
        let hub = self.hub.lock().await;
        let file = tokio::fs::File::open(local_path).await?;
        let mime_type = mime_guess::from_path(local_path)
            .first_or_octet_stream()
            .to_string();

        let result = hub
            .files()
            .create(google_drive3::api::File {
                name: Some(local_path.file_name().unwrap().to_str().unwrap().to_string()),
                parents: Some(vec![remote_path.to_string()]),
                mime_type: Some(mime_type),
                ..Default::default()
            })
            .upload(file, mime_type.parse().unwrap())
            .await
            .context("Failed to upload file")?;

        Ok(RemoteItem {
            id: result.1.id.unwrap_or_default(),
            name: result.1.name.unwrap_or_default(),
            path: format!("{}/{}", remote_path, result.1.name.unwrap_or_default()),
            is_dir: false,
            modified_at: chrono::DateTime::parse_from_rfc3339(&result.1.modified_time.unwrap_or_default())
                .unwrap_or_default()
                .into(),
            size: result.1.size.unwrap_or_default().parse().unwrap_or_default(),
        })
    }

    async fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<()> {
        let file_id = self.get_file_id(remote_path)
            .await?
            .context("File not found")?;

        let hub = self.hub.lock().await;
        let mut file = tokio::fs::File::create(local_path).await?;

        hub.files()
            .get(&file_id)
            .download(&mut file)
            .await
            .context("Failed to download file")?;

        Ok(())
    }

    async fn create_directory(&self, remote_path: &str) -> Result<RemoteItem> {
        let hub = self.hub.lock().await;
        let result = hub
            .files()
            .create(google_drive3::api::File {
                name: Some(Path::new(remote_path).file_name().unwrap().to_str().unwrap().to_string()),
                mime_type: Some("application/vnd.google-apps.folder".to_string()),
                ..Default::default()
            })
            .doit()
            .await
            .context("Failed to create directory")?;

        Ok(RemoteItem {
            id: result.1.id.unwrap_or_default(),
            name: result.1.name.unwrap_or_default(),
            path: remote_path.to_string(),
            is_dir: true,
            modified_at: chrono::DateTime::parse_from_rfc3339(&result.1.modified_time.unwrap_or_default())
                .unwrap_or_default()
                .into(),
            size: 0,
        })
    }

    async fn delete(&self, remote_path: &str) -> Result<()> {
        let file_id = self.get_file_id(remote_path)
            .await?
            .context("File not found")?;

        let hub = self.hub.lock().await;
        hub.files()
            .delete(&file_id)
            .doit()
            .await
            .context("Failed to delete file")?;

        Ok(())
    }

    async fn exists(&self, remote_path: &str) -> Result<bool> {
        Ok(self.get_file_id(remote_path).await?.is_some())
    }

    async fn get_item(&self, remote_path: &str) -> Result<Option<RemoteItem>> {
        let file_id = match self.get_file_id(remote_path).await? {
            Some(id) => id,
            None => return Ok(None),
        };

        let hub = self.hub.lock().await;
        let file = hub
            .files()
            .get(&file_id)
            .fields("id, name, mimeType, modifiedTime, size")
            .doit()
            .await
            .context("Failed to get file")?
            .1;

        Ok(Some(RemoteItem {
            id: file.id.unwrap_or_default(),
            name: file.name.unwrap_or_default(),
            path: remote_path.to_string(),
            is_dir: file.mime_type.unwrap_or_default() == "application/vnd.google-apps.folder",
            modified_at: chrono::DateTime::parse_from_rfc3339(&file.modified_time.unwrap_or_default())
                .unwrap_or_default()
                .into(),
            size: file.size.unwrap_or_default().parse().unwrap_or_default(),
        }))
    }

    async fn watch_local_changes(
        &self,
        local_path: &Path,
        tx: mpsc::Sender<ChangeType>,
    ) -> Result<()> {
        let (watcher_tx, mut watcher_rx) = mpsc::channel(100);
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
            if let Ok(event) = res {
                let _ = watcher_tx.blocking_send(event);
            }
        })?;

        watcher.watch(local_path, RecursiveMode::Recursive)?;

        tokio::spawn(async move {
            while let Some(event) = watcher_rx.recv().await {
                match event.kind {
                    notify::EventKind::Create(_) => {
                        for path in event.paths {
                            let _ = tx.send(ChangeType::Created(path)).await;
                        }
                    }
                    notify::EventKind::Modify(_) => {
                        for path in event.paths {
                            let _ = tx.send(ChangeType::Modified(path)).await;
                        }
                    }
                    notify::EventKind::Remove(_) => {
                        for path in event.paths {
                            let _ = tx.send(ChangeType::Deleted(path)).await;
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn watch_remote_changes(
        &self,
        remote_path: &str,
        tx: mpsc::Sender<RemoteItem>,
    ) -> Result<()> {
        // Google Drive API não tem um mecanismo de watch eficiente
        // Vamos fazer polling a cada 30 segundos
        let hub = self.hub.clone();
        let remote_path = remote_path.to_string();

        tokio::spawn(async move {
            let mut last_check = std::collections::HashMap::new();
            
            loop {
                let hub = hub.lock().await;
                let query = format!("'{}' in parents and trashed = false", remote_path);

                if let Ok(result) = hub
                    .files()
                    .list()
                    .q(&query)
                    .spaces("drive")
                    .fields("files(id, name, mimeType, modifiedTime, size)")
                    .doit()
                    .await
                {
                    if let Some(files) = result.1.files {
                        for file in files {
                            let id = file.id.clone().unwrap_or_default();
                            let modified = file.modified_time.clone().unwrap_or_default();

                            if let Some(last_modified) = last_check.get(&id) {
                                if modified != *last_modified {
                                    if let Ok(item) = RemoteItem::try_from(file.clone()) {
                                        let _ = tx.send(item).await;
                                    }
                                }
                            } else {
                                if let Ok(item) = RemoteItem::try_from(file.clone()) {
                                    let _ = tx.send(item).await;
                                }
                            }

                            last_check.insert(id, modified);
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });

        Ok(())
    }
}

impl TryFrom<google_drive3::api::File> for RemoteItem {
    type Error = anyhow::Error;

    fn try_from(file: google_drive3::api::File) -> Result<Self, Self::Error> {
        Ok(RemoteItem {
            id: file.id.unwrap_or_default(),
            name: file.name.unwrap_or_default(),
            path: format!("/{}", file.name.unwrap_or_default()),
            is_dir: file.mime_type.unwrap_or_default() == "application/vnd.google-apps.folder",
            modified_at: chrono::DateTime::parse_from_rfc3339(&file.modified_time.unwrap_or_default())
                .unwrap_or_default()
                .into(),
            size: file.size.unwrap_or_default().parse().unwrap_or_default(),
        })
    }
} 