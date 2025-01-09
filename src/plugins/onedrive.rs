use crate::plugins::Plugin;
use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::Bytes;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Arc,
};
use tokio::sync::RwLock;

const CREDENTIALS_PATH: &str = "onedrive_credentials.json";
const TOKEN_PATH: &str = "onedrive_token.json";
const GRAPH_API_URL: &str = "https://graph.microsoft.com/v1.0";

#[derive(Debug, Serialize, Deserialize)]
struct OneDriveCredentials {
    client_id: String,
    client_secret: String,
    tenant_id: String,
    redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OneDriveToken {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

#[derive(Clone)]
pub struct OneDrivePlugin {
    client: Arc<RwLock<Option<Client>>>,
    token: Arc<RwLock<Option<OneDriveToken>>>,
    root_folder: String,
}

impl OneDrivePlugin {
    pub fn new(root_folder: String) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            token: Arc::new(RwLock::new(None)),
            root_folder,
        }
    }

    async fn ensure_authenticated(&self) -> Result<()> {
        let mut client = self.client.write().await;
        let mut token = self.token.write().await;

        if client.is_none() || token.is_none() {
            let (new_client, new_token) = self.perform_oauth_flow().await?;
            *client = Some(new_client);
            *token = Some(new_token);
        }

        Ok(())
    }

    async fn perform_oauth_flow(&self) -> Result<(Client, OneDriveToken)> {
        // Try to load cached token
        if let Ok(token_data) = tokio::fs::read_to_string(TOKEN_PATH).await {
            if let Ok(token) = serde_json::from_str::<OneDriveToken>(&token_data) {
                let client = Client::builder()
                    .default_headers({
                        let mut headers = reqwest::header::HeaderMap::new();
                        headers.insert(
                            reqwest::header::AUTHORIZATION,
                            format!("Bearer {}", token.access_token).parse().unwrap(),
                        );
                        headers
                    })
                    .build()?;
                return Ok((client, token));
            }
        }

        // Load credentials
        let creds = tokio::fs::read_to_string(CREDENTIALS_PATH).await?;
        let creds: OneDriveCredentials = serde_json::from_str(&creds)?;

        // Create OAuth client
        let oauth_client = BasicClient::new(
            ClientId::new(creds.client_id.clone()),
            Some(ClientSecret::new(creds.client_secret.clone())),
            AuthUrl::new(format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                creds.tenant_id
            ))?,
            Some(TokenUrl::new(format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                creds.tenant_id
            ))?),
        )
        .set_redirect_uri(RedirectUrl::new(creds.redirect_uri)?);

        // Get authorization URL
        let (auth_url, _csrf_token) = oauth_client
            .authorize_url(oauth2::CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("Files.ReadWrite.All".to_string()))
            .add_scope(oauth2::Scope::new("offline_access".to_string()))
            .url();

        println!("Please visit this URL to authorize the application:");
        println!("{}", auth_url);
        println!("Enter the authorization code:");

        let mut code = String::new();
        std::io::stdin().read_line(&mut code)?;
        let code = oauth2::AuthorizationCode::new(code.trim().to_string());

        // Exchange code for token
        let token = oauth_client
            .exchange_code(code)
            .request_async(oauth2::reqwest::async_http_client)
            .await?;

        let onedrive_token = OneDriveToken {
            access_token: token.access_token().secret().clone(),
            refresh_token: token.refresh_token().map(|t| t.secret().clone()),
            expires_in: token.expires_in().unwrap_or_default().as_secs(),
        };

        // Cache the token
        tokio::fs::write(TOKEN_PATH, serde_json::to_string(&onedrive_token)?).await?;

        // Create HTTP client with auth header
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", onedrive_token.access_token)
                        .parse()
                        .unwrap(),
                );
                headers
            })
            .build()?;

        Ok((client, onedrive_token))
    }

    async fn create_upload_session(&self, path: &Path) -> Result<String> {
        self.ensure_authenticated().await?;
        let client = self.client.read().await;
        let client = client.as_ref().unwrap();

        let file_name = path.file_name().unwrap().to_string_lossy();
        let url = format!(
            "{}/me/drive/root:/{}/{}:/createUploadSession",
            GRAPH_API_URL, self.root_folder, file_name
        );

        let response = client.post(&url).send().await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json["uploadUrl"]
            .as_str()
            .context("Missing uploadUrl")?
            .to_string())
    }

    async fn upload_chunk_to_url(
        &self,
        url: &str,
        data: Bytes,
        offset: u64,
        total_size: u64,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let range_end = offset + data.len() as u64 - 1;
        let content_range = format!("bytes {}-{}/{}", offset, range_end, total_size);

        client
            .put(url)
            .header("Content-Range", content_range)
            .body(data)
            .send()
            .await?;

        Ok(())
    }

    async fn get_item_id(&self, path: &str) -> Result<String> {
        self.ensure_authenticated().await?;
        let client = self.client.read().await;
        let client = client.as_ref().unwrap();

        let url = format!(
            "{}/me/drive/root:/{}/{}",
            GRAPH_API_URL, self.root_folder, path
        );

        let response = client.get(&url).send().await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json["id"].as_str().context("Missing id")?.to_string())
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.ensure_authenticated().await?;
        let client = self.client.read().await;
        let client = client.as_ref().unwrap();

        let url = format!("{}/me/drive", GRAPH_API_URL);
        client.get(&url).send().await?;
        Ok(())
    }
}

#[async_trait]
impl Plugin for OneDrivePlugin {
    fn name(&self) -> &str {
        "onedrive"
    }

    async fn upload_chunk(&self, path: &Path, data: Bytes, offset: u64) -> Result<()> {
        self.ensure_authenticated().await?;

        // Create or get upload URL
        let upload_url = if offset == 0 {
            self.create_upload_session(path).await?
        } else {
            // TODO: Implement cache of upload URLs for subsequent chunks
            self.create_upload_session(path).await?
        };

        // Upload chunk
        self.upload_chunk_to_url(&upload_url, data.clone(), offset, data.len() as u64)
            .await?;

        Ok(())
    }

    async fn download_chunk(&self, path: &str, offset: u64, size: usize) -> Result<Bytes> {
        self.ensure_authenticated().await?;
        let client = self.client.read().await;
        let client = client.as_ref().unwrap();

        let item_id = self.get_item_id(path).await?;
        let url = format!("{}/me/drive/items/{}/content", GRAPH_API_URL, item_id);
        let range = format!("bytes={}-{}", offset, offset + size as u64 - 1);

        let response = client.get(&url).header("Range", range).send().await?;
        Ok(response.bytes().await?)
    }

    async fn delete_file(&self, path: &Path) -> Result<()> {
        self.ensure_authenticated().await?;
        let client = self.client.read().await;
        let client = client.as_ref().unwrap();

        let item_id = self
            .get_item_id(&path.file_name().unwrap().to_string_lossy())
            .await?;

        let url = format!("{}/me/drive/items/{}", GRAPH_API_URL, item_id);
        client.delete(&url).send().await?;

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_onedrive_plugin() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test data").await?;

        let plugin = OneDrivePlugin::new("root".to_string());

        // Test upload in chunks
        let data = Bytes::from("test data");
        plugin.upload_chunk(&test_file, data.clone(), 0).await?;

        // Test download in chunks
        let downloaded = plugin.download_chunk("test.txt", 0, data.len()).await?;
        assert_eq!(downloaded, data);

        // Test deletion
        plugin.delete_file(&test_file).await?;

        Ok(())
    }
}
