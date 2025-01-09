use anyhow::Result;
use bytes::Bytes;
use filesynchub::plugins::{google_drive::GoogleDriveClient, Plugin};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize Google Drive client
    let client = GoogleDriveClient::new("FileSyncHub".to_string()).await?;
    println!("Google Drive client initialized successfully!");

    // Create a test file
    let test_file = PathBuf::from("test.txt");
    tokio::fs::write(&test_file, b"Hello, Google Drive!").await?;

    // Upload the file
    println!("Uploading file...");
    client.upload_chunk(&test_file, Bytes::from("Hello, Google Drive!"), 0).await?;
    println!("File uploaded successfully!");

    // Clean up
    tokio::fs::remove_file(test_file).await?;

    Ok(())
}
