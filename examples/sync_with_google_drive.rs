use anyhow::Result;
use bytes::Bytes;
use filesynchub::plugins::google_drive::GoogleDrivePlugin;
use filesynchub::plugins::Plugin;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new Google Drive plugin instance
    let plugin = GoogleDrivePlugin::new("FileSyncHub".to_string());

    // Test the connection
    println!("Testing connection to Google Drive...");
    plugin.test_connection().await?;
    println!("Connection successful!");

    // Create a test file
    let test_file_path = Path::new("test.txt");
    let test_data = b"Hello from FileSyncHub!";
    tokio::fs::write(test_file_path, test_data).await?;

    // Upload the file
    println!("Uploading test file...");
    plugin
        .upload_chunk(test_file_path, Bytes::from(&test_data[..]), 0)
        .await?;
    println!("Upload successful!");

    // Download the file
    println!("Downloading test file...");
    let downloaded = plugin
        .download_chunk("test.txt", 0, test_data.len())
        .await?;
    println!("Download successful!");

    // Verify the content
    assert_eq!(&downloaded[..], &test_data[..]);
    println!("Content verification successful!");

    // Clean up
    println!("Deleting test file...");
    plugin.delete_file(test_file_path).await?;
    println!("Deletion successful!");

    tokio::fs::remove_file(test_file_path).await?;
    println!("Local file cleaned up!");

    Ok(())
}
