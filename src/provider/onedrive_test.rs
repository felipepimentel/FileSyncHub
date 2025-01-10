use super::*;
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::mpsc;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_onedrive_provider() -> Result<()> {
    // Iniciar servidor mock
    let mock_server = MockServer::start().await;

    // Configurar mocks para as chamadas da API
    Mock::given(method("GET"))
        .and(path("/me/drive/root:/test:/children"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "value": [
                {
                    "id": "123",
                    "name": "test.txt",
                    "lastModifiedDateTime": "2023-01-01T00:00:00Z",
                    "size": 100
                },
                {
                    "id": "456",
                    "name": "folder",
                    "lastModifiedDateTime": "2023-01-01T00:00:00Z",
                    "folder": {}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // Criar provedor com servidor mock
    let provider = OneDriveProvider::new(
        "test".to_string(),
        "client_id".to_string(),
        "client_secret".to_string(),
        Some("test_token".to_string()),
    )?;

    // Testar listagem de arquivos
    let files = provider.list_files("/test").await?;
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].name, "test.txt");
    assert!(!files[0].is_dir);
    assert_eq!(files[1].name, "folder");
    assert!(files[1].is_dir);

    // Testar upload de arquivo
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("test.txt");
    tokio::fs::write(&test_file, b"test content").await?;

    Mock::given(method("PUT"))
        .and(path("/me/drive/root:/test/upload.txt:/content"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "789",
            "name": "upload.txt",
            "lastModifiedDateTime": "2023-01-01T00:00:00Z",
            "size": 12
        })))
        .mount(&mock_server)
        .await;

    let uploaded = provider.upload_file(&test_file, "/test/upload.txt").await?;
    assert_eq!(uploaded.name, "upload.txt");
    assert_eq!(uploaded.size, 12);

    // Testar download de arquivo
    let download_path = temp_dir.path().join("downloaded.txt");

    Mock::given(method("GET"))
        .and(path("/me/drive/root:/test/download.txt:/content"))
        .respond_with(ResponseTemplate::new(200).set_body("test content"))
        .mount(&mock_server)
        .await;

    provider.download_file("/test/download.txt", &download_path).await?;
    let content = tokio::fs::read_to_string(&download_path).await?;
    assert_eq!(content, "test content");

    // Testar criação de diretório
    Mock::given(method("POST"))
        .and(path("/me/drive/root:/test:/children"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "999",
            "name": "new_folder",
            "lastModifiedDateTime": "2023-01-01T00:00:00Z",
            "folder": {}
        })))
        .mount(&mock_server)
        .await;

    let created = provider.create_directory("/test/new_folder").await?;
    assert_eq!(created.name, "new_folder");
    assert!(created.is_dir);

    // Testar deleção
    Mock::given(method("DELETE"))
        .and(path("/me/drive/root:/test/delete.txt:"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    provider.delete("/test/delete.txt").await?;

    // Testar monitoramento de mudanças locais
    let (tx, mut rx) = mpsc::channel(100);
    provider.watch_local_changes(temp_dir.path(), tx).await?;

    // Criar arquivo para testar o watcher
    let watch_file = temp_dir.path().join("watch.txt");
    tokio::fs::write(&watch_file, b"test").await?;

    // Esperar pela notificação
    let timeout = tokio::time::sleep(Duration::from_secs(1));
    tokio::pin!(timeout);

    tokio::select! {
        Some(change) = rx.recv() => {
            match change {
                ChangeType::Created(path) => assert_eq!(path, watch_file),
                _ => panic!("Unexpected change type"),
            }
        }
        _ = &mut timeout => panic!("Timeout waiting for file change"),
    }

    Ok(())
}

#[tokio::test]
async fn test_onedrive_error_handling() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Configurar mock para erro de autenticação
    Mock::given(method("GET"))
        .and(path("/me/drive/root:/test:/children"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "code": "InvalidAuthenticationToken",
                "message": "Access token is invalid"
            }
        })))
        .mount(&mock_server)
        .await;

    let provider = OneDriveProvider::new(
        "test".to_string(),
        "client_id".to_string(),
        "client_secret".to_string(),
        Some("invalid_token".to_string()),
    )?;

    // Verificar se o erro é propagado corretamente
    let result = provider.list_files("/test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API request failed"));

    Ok(())
}

#[tokio::test]
async fn test_onedrive_rate_limiting() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Configurar mock para simular rate limiting
    Mock::given(method("GET"))
        .and(path("/me/drive/root:/test:/children"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {
                "code": "TooManyRequests",
                "message": "Too many requests"
            }
        })))
        .mount(&mock_server)
        .await;

    let provider = OneDriveProvider::new(
        "test".to_string(),
        "client_id".to_string(),
        "client_secret".to_string(),
        Some("test_token".to_string()),
    )?;

    // Verificar se o erro de rate limiting é propagado
    let result = provider.list_files("/test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API request failed"));

    Ok(())
} 