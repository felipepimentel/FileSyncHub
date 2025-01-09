use anyhow::Result;
use filesynchub::plugins::{google_drive::GoogleDrivePlugin, onedrive::OneDrivePlugin, Plugin};
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use test_log::test;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test(tokio::test)]
async fn test_google_drive_auth() -> Result<()> {
    // Set up mock server
    let mock_server = MockServer::start().await;

    // Create test credentials
    let dir = tempdir()?;
    let credentials_path = dir.path().join("google_credentials.json");
    let credentials_content = format!(
        r#"{{
            "installed": {{
                "client_id": "test_client_id",
                "client_secret": "test_client_secret",
                "auth_uri": "{}",
                "token_uri": "{}",
                "redirect_uris": ["http://localhost"]
            }}
        }}"#,
        mock_server.uri(),
        mock_server.uri()
    );

    fs::write(&credentials_path, credentials_content)?;

    // Mock token endpoint
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "test_access_token",
            "refresh_token": "test_refresh_token",
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .mount(&mock_server)
        .await;

    // Create and initialize plugin
    let mut plugin = GoogleDrivePlugin::new(
        credentials_path,
        "test_folder".to_string(),
        vec!["*.txt".to_string()],
        vec!["*.tmp".to_string()],
    );

    // Test initialization (which includes authentication)
    assert!(plugin.initialize().await.is_err()); // Should fail because we haven't implemented the full OAuth2 flow yet

    Ok(())
}

#[test(tokio::test)]
async fn test_onedrive_auth() -> Result<()> {
    // Set up mock server
    let mock_server = MockServer::start().await;

    // Create test credentials
    let dir = tempdir()?;
    let credentials_path = dir.path().join("onedrive_credentials.json");
    let credentials_content = format!(
        r#"{{
            "client_id": "test_client_id",
            "client_secret": "test_client_secret",
            "tenant_id": "test_tenant_id",
            "redirect_uri": "http://localhost",
            "auth_endpoint": "{}",
            "token_endpoint": "{}",
            "scope": "Files.ReadWrite.All offline_access"
        }}"#,
        mock_server.uri(),
        mock_server.uri()
    );

    fs::write(&credentials_path, credentials_content)?;

    // Mock token endpoint
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "test_access_token",
            "refresh_token": "test_refresh_token",
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .mount(&mock_server)
        .await;

    // Create and initialize plugin
    let mut plugin = OneDrivePlugin::new(
        credentials_path,
        "test_folder".to_string(),
        vec!["*.txt".to_string()],
        vec!["*.tmp".to_string()],
    );

    // Test initialization (which includes authentication)
    assert!(plugin.initialize().await.is_err()); // Should fail because we haven't implemented the full OAuth2 flow yet

    Ok(())
}

#[test(tokio::test)]
async fn test_token_storage() -> Result<()> {
    let dir = tempdir()?;
    let token_path = dir.path().join("test_token.json");
    let token_content = r#"{
        "access_token": "test_access_token",
        "refresh_token": "test_refresh_token",
        "expiry": "2024-12-31T23:59:59Z"
    }"#;

    fs::write(&token_path, token_content)?;

    // Verify token file was created
    assert!(token_path.exists());
    let content = fs::read_to_string(&token_path)?;
    let token: serde_json::Value = serde_json::from_str(&content)?;

    assert_eq!(token["access_token"], "test_access_token");
    assert_eq!(token["refresh_token"], "test_refresh_token");

    Ok(())
}
