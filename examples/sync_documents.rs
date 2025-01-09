use anyhow::Result;
use bytes::Bytes;
use filesynchub::plugins::{google_drive::GoogleDriveClient, onedrive::OneDrivePlugin, Plugin};
use std::path::PathBuf;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize plugins
    let google_drive = GoogleDriveClient::new("Documents".to_string()).await?;
    let onedrive = OneDrivePlugin::new("Documents".to_string());

    // Create test directory
    let test_dir = PathBuf::from("test_documents");
    fs::create_dir_all(&test_dir).await?;

    // Create some test files
    let files = vec!["doc1.txt", "doc2.txt", "doc3.txt"];
    for (i, file) in files.iter().enumerate() {
        let file_path = test_dir.join(file);
        fs::write(&file_path, format!("Document {}", i + 1)).await?;

        // Upload to Google Drive
        println!("Uploading {} to Google Drive...", file);
        google_drive
            .upload_chunk(&file_path, Bytes::from(format!("Document {}", i + 1)), 0)
            .await?;

        // Upload to OneDrive
        println!("Uploading {} to OneDrive...", file);
        onedrive
            .upload_chunk(&file_path, Bytes::from(format!("Document {}", i + 1)), 0)
            .await?;
    }

    // Clean up
    fs::remove_dir_all(test_dir).await?;
    println!("Sync completed successfully!");

    Ok(())
}
