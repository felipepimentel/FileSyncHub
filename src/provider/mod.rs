use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;

pub mod factory;
pub mod google_drive;
pub mod onedrive;

#[derive(Debug, Clone)]
pub struct RemoteItem {
    pub id: String,
    pub name: String,
    pub is_folder: bool,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Created(std::path::PathBuf),
    Modified(std::path::PathBuf),
    Deleted(std::path::PathBuf),
}

pub use crate::config::FolderMapping;

#[async_trait]
pub trait CloudProvider: Send + Sync {
    async fn list_files(&self, remote_path: &str) -> Result<Vec<RemoteItem>>;
    async fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<RemoteItem>;
    async fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<()>;
    async fn create_directory(&self, remote_path: &str) -> Result<RemoteItem>;
    async fn delete(&self, remote_path: &str) -> Result<()>;
    async fn exists(&self, remote_path: &str) -> Result<bool>;
    async fn get_item(&self, remote_path: &str) -> Result<Option<RemoteItem>>;
    async fn watch_local_changes(&self, local_path: &Path, tx: mpsc::Sender<ChangeType>) -> Result<()>;
    async fn watch_remote_changes(&self, remote_path: &str, tx: mpsc::Sender<RemoteItem>) -> Result<()>;
    async fn get_mappings(&self) -> Vec<FolderMapping>;
} 