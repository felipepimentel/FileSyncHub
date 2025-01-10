use async_trait::async_trait;
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

/// Representa um arquivo ou diretório remoto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub size: u64,
}

/// Representa uma mudança detectada
#[derive(Debug, Clone)]
pub enum ChangeType {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

/// Interface para provedores de armazenamento em nuvem
#[async_trait]
pub trait CloudProvider: Send + Sync {
    /// Inicializa o provedor com as credenciais necessárias
    async fn initialize(&mut self) -> Result<()>;

    /// Lista arquivos em um diretório remoto
    async fn list_files(&self, remote_path: &str) -> Result<Vec<RemoteItem>>;

    /// Faz upload de um arquivo
    async fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<RemoteItem>;

    /// Faz download de um arquivo
    async fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<()>;

    /// Cria um diretório remoto
    async fn create_directory(&self, remote_path: &str) -> Result<RemoteItem>;

    /// Remove um arquivo ou diretório remoto
    async fn delete(&self, remote_path: &str) -> Result<()>;

    /// Verifica se um arquivo ou diretório existe remotamente
    async fn exists(&self, remote_path: &str) -> Result<bool>;

    /// Obtém informações sobre um arquivo ou diretório remoto
    async fn get_item(&self, remote_path: &str) -> Result<Option<RemoteItem>>;

    /// Monitora mudanças em um diretório local
    async fn watch_local_changes(
        &self,
        local_path: &Path,
        tx: mpsc::Sender<ChangeType>,
    ) -> Result<()>;

    /// Monitora mudanças remotas
    async fn watch_remote_changes(
        &self,
        remote_path: &str,
        tx: mpsc::Sender<RemoteItem>,
    ) -> Result<()>;
}

/// Factory para criar instâncias de provedores
#[async_trait]
pub trait ProviderFactory: Send + Sync {
    /// Cria uma nova instância do provedor
    async fn create_provider(&self, config: &crate::config::ProviderConfig) -> Result<Box<dyn CloudProvider>>;
}

// Re-export módulos específicos de provedores
pub mod google_drive;
pub mod onedrive;

// Re-export factory de provedores
mod factory;
pub use factory::create_provider; 