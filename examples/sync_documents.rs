use anyhow::Result;
use bytes::Bytes;
use filesynchub::plugins::{google_drive::GoogleDrivePlugin, onedrive::OneDrivePlugin, Plugin};
use std::{path::Path, time::Duration};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Create plugins
    let google_drive = GoogleDrivePlugin::new("FileSyncHub/Documents".to_string());
    let onedrive = OneDrivePlugin::new("FileSyncHub/Documents".to_string());

    // Test connections
    println!("Testing connections...");
    google_drive.test_connection().await?;
    onedrive.test_connection().await?;
    println!("Connections successful!");

    // Create test documents
    let docs_dir = Path::new("test_documents");
    fs::create_dir_all(docs_dir).await?;

    // Create a text document
    let text_file = docs_dir.join("notes.txt");
    let text_content = "Important notes from FileSyncHub!";
    fs::write(&text_file, text_content).await?;

    // Create a markdown document
    let md_file = docs_dir.join("readme.md");
    let md_content = "# FileSyncHub Test\n\nThis is a test markdown file.";
    fs::write(&md_file, md_content).await?;

    // Sync text document to Google Drive
    println!("Syncing text document to Google Drive...");
    google_drive
        .upload_chunk(&text_file, Bytes::from(text_content), 0)
        .await?;
    println!("Text document synced to Google Drive!");

    // Sync markdown document to OneDrive
    println!("Syncing markdown document to OneDrive...");
    onedrive
        .upload_chunk(&md_file, Bytes::from(md_content), 0)
        .await?;
    println!("Markdown document synced to OneDrive!");

    // Wait a moment
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Download and verify documents
    println!("Verifying documents...");

    // Verify Google Drive document
    let downloaded_text = google_drive
        .download_chunk("notes.txt", 0, text_content.len())
        .await?;
    assert_eq!(downloaded_text, Bytes::from(text_content));
    println!("Google Drive document verified!");

    // Verify OneDrive document
    let downloaded_md = onedrive
        .download_chunk("readme.md", 0, md_content.len())
        .await?;
    assert_eq!(downloaded_md, Bytes::from(md_content));
    println!("OneDrive document verified!");

    // Clean up
    println!("Cleaning up...");
    google_drive.delete_file(&text_file).await?;
    onedrive.delete_file(&md_file).await?;
    fs::remove_dir_all(docs_dir).await?;
    println!("Cleanup complete!");

    Ok(())
}
