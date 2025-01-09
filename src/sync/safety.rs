use anyhow::Result;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

const MAX_BACKUPS: usize = 5;

#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub hash: String,
    pub modified: DateTime<Utc>,
    pub size: u64,
    pub backup_path: Option<PathBuf>,
}

pub struct SafeSync {
    metadata_cache: Arc<RwLock<HashMap<PathBuf, FileMetadata>>>,
    backup_dir: PathBuf,
}

impl SafeSync {
    pub fn new(backup_dir: PathBuf) -> Self {
        Self {
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            backup_dir,
        }
    }

    /// Verifica se é seguro sincronizar um arquivo
    pub async fn is_safe_to_sync(&self, path: &Path) -> Result<bool> {
        let metadata = tokio::fs::metadata(path).await?;
        let modified = metadata.modified()?.into();
        let size = metadata.len();

        let hash = self.calculate_hash(path).await?;

        // Verificar se o arquivo existe no cache
        if let Some(cached) = self.metadata_cache.read().await.get(path) {
            // Se o arquivo foi modificado, verificar se temos backup
            if cached.modified != modified || cached.size != size || cached.hash != hash {
                if let Some(backup_path) = &cached.backup_path {
                    // Verificar integridade do backup
                    if self.verify_backup(backup_path, &cached.hash).await? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
        }

        // Se o arquivo não existe no cache, criar backup
        self.create_backup(path, &hash).await?;
        Ok(true)
    }

    /// Calcula o hash SHA-256 de um arquivo
    async fn calculate_hash(&self, path: &Path) -> Result<String> {
        let mut file = File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Cria um backup do arquivo
    async fn create_backup(&self, path: &Path, hash: &str) -> Result<()> {
        // Criar diretório de backup se não existir
        tokio::fs::create_dir_all(&self.backup_dir).await?;

        let backup_path = self.backup_dir.join(format!(
            "{}_{}",
            path.file_name().unwrap().to_string_lossy(),
            hash
        ));

        // Copiar arquivo para backup
        tokio::fs::copy(path, &backup_path).await?;

        // Atualizar cache
        let metadata = tokio::fs::metadata(path).await?;
        let file_metadata = FileMetadata {
            hash: hash.to_string(),
            modified: metadata.modified()?.into(),
            size: metadata.len(),
            backup_path: Some(backup_path),
        };

        self.metadata_cache
            .write()
            .await
            .insert(path.to_path_buf(), file_metadata);

        // Limpar backups antigos
        self.cleanup_old_backups(path).await?;

        Ok(())
    }

    /// Verifica a integridade de um backup
    async fn verify_backup(&self, backup_path: &Path, expected_hash: &str) -> Result<bool> {
        let hash = self.calculate_hash(backup_path).await?;
        Ok(hash == expected_hash)
    }

    /// Restaura um arquivo do backup mais recente
    pub async fn restore_from_backup(&self, path: &Path) -> Result<()> {
        if let Some(metadata) = self.metadata_cache.read().await.get(path) {
            if let Some(backup_path) = &metadata.backup_path {
                if self.verify_backup(backup_path, &metadata.hash).await? {
                    tokio::fs::copy(backup_path, path).await?;
                    return Ok(());
                }
            }
        }
        Err(anyhow::anyhow!("No valid backup found"))
    }

    /// Remove backups antigos mantendo apenas os MAX_BACKUPS mais recentes
    async fn cleanup_old_backups(&self, path: &Path) -> Result<()> {
        let filename = path.file_name().unwrap().to_string_lossy();
        let mut backups: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&filename.to_string())
            })
            .collect();

        // Ordenar por data de modificação (mais recente primeiro)
        backups
            .sort_by_key(|entry| std::cmp::Reverse(entry.metadata().unwrap().modified().unwrap()));

        // Remover backups excedentes
        for entry in backups.iter().skip(MAX_BACKUPS) {
            fs::remove_file(entry.path())?;
        }

        Ok(())
    }

    /// Verifica a integridade de um arquivo
    pub async fn verify_file_integrity(&self, path: &Path) -> Result<bool> {
        if let Some(metadata) = self.metadata_cache.read().await.get(path) {
            let current_hash = self.calculate_hash(path).await?;
            Ok(current_hash == metadata.hash)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_safe_sync() -> Result<()> {
        let temp_dir = tempdir()?;
        let backup_dir = temp_dir.path().join("backups");
        let safe_sync = SafeSync::new(backup_dir);

        // Criar arquivo de teste
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test data").await?;

        // Verificar se é seguro sincronizar
        assert!(safe_sync.is_safe_to_sync(&test_file).await?);

        // Modificar arquivo
        tokio::fs::write(&test_file, b"modified data").await?;

        // Verificar que não é seguro sincronizar sem backup
        assert!(!safe_sync.is_safe_to_sync(&test_file).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_backup_restore() -> Result<()> {
        let temp_dir = tempdir()?;
        let backup_dir = temp_dir.path().join("backups");
        let safe_sync = SafeSync::new(backup_dir);

        // Criar arquivo de teste
        let test_file = temp_dir.path().join("test.txt");
        let original_data = b"test data";
        tokio::fs::write(&test_file, original_data).await?;

        // Criar backup
        assert!(safe_sync.is_safe_to_sync(&test_file).await?);

        // Modificar arquivo
        tokio::fs::write(&test_file, b"modified data").await?;

        // Restaurar do backup
        safe_sync.restore_from_backup(&test_file).await?;

        // Verificar conteúdo restaurado
        let restored_data = tokio::fs::read(&test_file).await?;
        assert_eq!(restored_data, original_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_backup_cleanup() -> Result<()> {
        let temp_dir = tempdir()?;
        let backup_dir = temp_dir.path().join("backups");
        let safe_sync = SafeSync::new(backup_dir.clone());

        // Criar arquivo de teste
        let test_file = temp_dir.path().join("test.txt");

        // Criar múltiplos backups
        for i in 0..MAX_BACKUPS + 2 {
            tokio::fs::write(&test_file, format!("data {}", i)).await?;
            safe_sync.is_safe_to_sync(&test_file).await?;
        }

        // Verificar número de backups
        let backup_count = fs::read_dir(&backup_dir)?
            .filter_map(|entry| entry.ok())
            .count();
        assert_eq!(backup_count, MAX_BACKUPS);

        Ok(())
    }
}
