use anyhow::{Result, Context};
use async_trait::async_trait;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    AuthorizationCode, TokenResponse,
};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use notify::{Watcher, RecursiveMode};
use std::sync::Arc;
use tokio::sync::Mutex;
use super::{CloudProvider, RemoteItem, ChangeType};

const MICROSOFT_GRAPH_URL: &str = "https://graph.microsoft.com/v1.0";
const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

#[derive(Debug, Serialize, Deserialize)]
struct OneDriveItem {
    id: String,
    name: String,
    #[serde(rename = "lastModifiedDateTime")]
    last_modified: String,
    size: Option<i64>,
    #[serde(rename = "folder")]
    is_folder: Option<serde_json::Value>,
}

pub struct OneDriveProvider {
    name: String,
    client: Arc<Client>,
    oauth: Arc<Mutex<BasicClient>>,
    access_token: Arc<Mutex<Option<String>>>,
}

impl OneDriveProvider {
    pub fn new(
        name: String,
        client_id: String,
        client_secret: String,
        token: Option<String>,
    ) -> Result<Self> {
        let oauth = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(AUTH_URL.to_string())?,
            Some(TokenUrl::new(TOKEN_URL.to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new("http://localhost:8080".to_string())?);

        let client = Client::builder()
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            name,
            client: Arc::new(client),
            oauth: Arc::new(Mutex::new(oauth)),
            access_token: Arc::new(Mutex::new(token)),
        })
    }

    async fn ensure_token(&self) -> Result<String> {
        let token = self.access_token.lock().await;
        if let Some(token) = token.as_ref() {
            return Ok(token.clone());
        }
        
        // TODO: Implementar refresh token
        anyhow::bail!("No access token available")
    }

    async fn api_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<T> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", MICROSOFT_GRAPH_URL, path);

        let mut request = self.client
            .request(method, &url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token));

        if let Some(body) = body {
            request = request
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .body(body);
        }

        let response = request
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("API request failed: {}", error);
        }

        response.json().await.context("Failed to parse response")
    }

    async fn get_item_by_path(&self, path: &str) -> Result<Option<OneDriveItem>> {
        let encoded_path = urlencoding::encode(path);
        let result: Result<OneDriveItem> = self.api_request(
            reqwest::Method::GET,
            &format!("/me/drive/root:{}:", &encoded_path),
            None,
        ).await;

        match result {
            Ok(item) => Ok(Some(item)),
            Err(e) => {
                if e.to_string().contains("404") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[async_trait]
impl CloudProvider for OneDriveProvider {
    async fn initialize(&mut self) -> Result<()> {
        // TODO: Implementar fluxo de autorização se não tiver token
        Ok(())
    }

    async fn list_files(&self, remote_path: &str) -> Result<Vec<RemoteItem>> {
        let encoded_path = urlencoding::encode(remote_path);
        let response: serde_json::Value = self.api_request(
            reqwest::Method::GET,
            &format!("/me/drive/root:{}:/children", &encoded_path),
            None,
        ).await?;

        let mut items = Vec::new();
        if let Some(values) = response["value"].as_array() {
            for value in values {
                let item: OneDriveItem = serde_json::from_value(value.clone())?;
                items.push(RemoteItem {
                    id: item.id,
                    name: item.name.clone(),
                    path: format!("{}/{}", remote_path, item.name),
                    is_dir: item.is_folder.is_some(),
                    modified_at: chrono::DateTime::parse_from_rfc3339(&item.last_modified)?.into(),
                    size: item.size.unwrap_or_default() as u64,
                });
            }
        }

        Ok(items)
    }

    async fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<RemoteItem> {
        let content = tokio::fs::read(local_path).await?;
        let encoded_path = urlencoding::encode(remote_path);
        
        let response: OneDriveItem = self.api_request(
            reqwest::Method::PUT,
            &format!("/me/drive/root:{}:/content", &encoded_path),
            Some(content),
        ).await?;

        Ok(RemoteItem {
            id: response.id,
            name: response.name,
            path: remote_path.to_string(),
            is_dir: false,
            modified_at: chrono::DateTime::parse_from_rfc3339(&response.last_modified)?.into(),
            size: response.size.unwrap_or_default() as u64,
        })
    }

    async fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<()> {
        let encoded_path = urlencoding::encode(remote_path);
        let token = self.ensure_token().await?;
        
        let response = self.client
            .get(&format!("{}/me/drive/root:{}:/content", MICROSOFT_GRAPH_URL, &encoded_path))
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let content = response.bytes().await?;
        tokio::fs::write(local_path, content).await?;

        Ok(())
    }

    async fn create_directory(&self, remote_path: &str) -> Result<RemoteItem> {
        let encoded_path = urlencoding::encode(remote_path);
        let name = Path::new(remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid directory name")?;

        let parent_path = Path::new(remote_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        let response: OneDriveItem = self.api_request(
            reqwest::Method::POST,
            &format!("/me/drive/root:{}:/children", &urlencoding::encode(parent_path)),
            Some(serde_json::json!({
                "name": name,
                "folder": {},
                "@microsoft.graph.conflictBehavior": "rename"
            }).to_string().into_bytes()),
        ).await?;

        Ok(RemoteItem {
            id: response.id,
            name: response.name,
            path: remote_path.to_string(),
            is_dir: true,
            modified_at: chrono::DateTime::parse_from_rfc3339(&response.last_modified)?.into(),
            size: 0,
        })
    }

    async fn delete(&self, remote_path: &str) -> Result<()> {
        let encoded_path = urlencoding::encode(remote_path);
        let _: serde_json::Value = self.api_request(
            reqwest::Method::DELETE,
            &format!("/me/drive/root:{}:", &encoded_path),
            None,
        ).await?;

        Ok(())
    }

    async fn exists(&self, remote_path: &str) -> Result<bool> {
        Ok(self.get_item_by_path(remote_path).await?.is_some())
    }

    async fn get_item(&self, remote_path: &str) -> Result<Option<RemoteItem>> {
        let item = match self.get_item_by_path(remote_path).await? {
            Some(item) => item,
            None => return Ok(None),
        };

        Ok(Some(RemoteItem {
            id: item.id,
            name: item.name,
            path: remote_path.to_string(),
            is_dir: item.is_folder.is_some(),
            modified_at: chrono::DateTime::parse_from_rfc3339(&item.last_modified)?.into(),
            size: item.size.unwrap_or_default() as u64,
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
        // OneDrive também não tem um mecanismo eficiente de watch
        // Vamos usar polling como no Google Drive
        let client = self.client.clone();
        let access_token = self.access_token.clone();
        let remote_path = remote_path.to_string();

        tokio::spawn(async move {
            let mut last_check = std::collections::HashMap::new();
            
            loop {
                if let Ok(token) = access_token.lock().await.clone() {
                    if let Some(token) = token {
                        let url = format!(
                            "{}/me/drive/root:{}:/children",
                            MICROSOFT_GRAPH_URL,
                            urlencoding::encode(&remote_path)
                        );

                        if let Ok(response) = client
                            .get(&url)
                            .header(header::AUTHORIZATION, format!("Bearer {}", token))
                            .send()
                            .await
                        {
                            if let Ok(json) = response.json::<serde_json::Value>().await {
                                if let Some(items) = json["value"].as_array() {
                                    for item in items {
                                        if let Ok(item) = serde_json::from_value::<OneDriveItem>(item.clone()) {
                                            let id = item.id.clone();
                                            let modified = item.last_modified.clone();

                                            if let Some(last_modified) = last_check.get(&id) {
                                                if modified != *last_modified {
                                                    let remote_item = RemoteItem {
                                                        id: item.id,
                                                        name: item.name,
                                                        path: format!("{}/{}", remote_path, item.name),
                                                        is_dir: item.is_folder.is_some(),
                                                        modified_at: chrono::DateTime::parse_from_rfc3339(&item.last_modified)
                                                            .unwrap_or_default()
                                                            .into(),
                                                        size: item.size.unwrap_or_default() as u64,
                                                    };
                                                    let _ = tx.send(remote_item).await;
                                                }
                                            }

                                            last_check.insert(id, modified);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });

        Ok(())
    }
} 