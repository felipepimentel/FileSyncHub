mod performance;
mod safety;

use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;

use crate::plugins::Plugin;
use performance::PerformanceOptimizer;
use safety::SafeSync;

pub struct SyncManager {
    plugins: Vec<Box<dyn Plugin>>,
    safe_sync: Arc<SafeSync>,
    performance: Arc<PerformanceOptimizer>,
    temp_dir: PathBuf,
}

impl SyncManager {
    pub fn new(temp_dir: PathBuf) -> Self {
        let backup_dir = temp_dir.join("backups");
        Self {
            plugins: Vec::new(),
            safe_sync: Arc::new(SafeSync::new(backup_dir)),
            performance: Arc::new(PerformanceOptimizer::new(temp_dir.clone())),
            temp_dir,
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Sincroniza um arquivo com todos os plugins registrados
    pub async fn sync_file(&self, path: &Path) -> Result<()> {
        // Verificar se é seguro sincronizar
        if !self.safe_sync.is_safe_to_sync(path).await? {
            log::warn!("Não é seguro sincronizar o arquivo: {}", path.display());
            return Ok(());
        }

        // Processar arquivo em chunks para otimização
        let chunks = self.performance.process_file(path).await?;

        // Sincronizar com cada plugin
        for plugin in &self.plugins {
            let plugin_name = plugin.name();
            log::info!(
                "Sincronizando {} com plugin {}",
                path.display(),
                plugin_name
            );

            // Upload do arquivo em chunks
            self.performance
                .upload_chunks(path, |data, offset| {
                    let plugin = plugin.clone();
                    Box::pin(async move {
                        plugin.upload_chunk(path, data, offset).await?;
                        Ok(())
                    })
                })
                .await?;

            log::info!(
                "Sincronização completa de {} com plugin {}",
                path.display(),
                plugin_name
            );
        }

        Ok(())
    }

    /// Baixa um arquivo de um plugin específico
    pub async fn download_file(
        &self,
        plugin: &Box<dyn Plugin>,
        remote_path: &str,
        local_path: &Path,
        size: u64,
    ) -> Result<()> {
        log::info!(
            "Baixando arquivo {} do plugin {}",
            remote_path,
            plugin.name()
        );

        // Download do arquivo em chunks
        self.performance
            .download_chunks(local_path, size, |offset, chunk_size| {
                let plugin = plugin.clone();
                let remote_path = remote_path.to_string();
                Box::pin(async move {
                    plugin
                        .download_chunk(&remote_path, offset, chunk_size)
                        .await
                })
            })
            .await?;

        // Verificar integridade do arquivo baixado
        if !self.safe_sync.verify_file_integrity(local_path).await? {
            log::error!(
                "Falha na verificação de integridade do arquivo baixado: {}",
                local_path.display()
            );
            // Tentar restaurar do backup se disponível
            self.safe_sync.restore_from_backup(local_path).await?;
        }

        log::info!(
            "Download completo de {} do plugin {}",
            remote_path,
            plugin.name()
        );

        Ok(())
    }

    /// Deleta um arquivo localmente e em todos os plugins
    pub async fn delete_file(&self, path: &Path) -> Result<()> {
        // Criar backup antes de deletar
        if !self.safe_sync.is_safe_to_sync(path).await? {
            log::warn!(
                "Não é seguro deletar o arquivo sem backup: {}",
                path.display()
            );
            return Ok(());
        }

        // Deletar em cada plugin
        for plugin in &self.plugins {
            plugin.delete_file(path).await?;
        }

        // Deletar arquivo local
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }

        Ok(())
    }

    /// Limpa os caches e arquivos temporários
    pub async fn cleanup(&self) -> Result<()> {
        self.performance.clear_cache().await;

        // Limpar diretório temporário
        if self.temp_dir.exists() {
            tokio::fs::remove_dir_all(&self.temp_dir).await?;
            tokio::fs::create_dir_all(&self.temp_dir).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use bytes::Bytes;
    use tempfile::tempdir;

    #[derive(Clone)]
    struct MockPlugin {
        name: String,
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        async fn upload_chunk(&self, path: &Path, data: Bytes, offset: u64) -> Result<()> {
            log::info!(
                "Mock upload: {} offset={} size={}",
                path.display(),
                offset,
                data.len()
            );
            Ok(())
        }

        async fn download_chunk(&self, path: &str, offset: u64, size: usize) -> Result<Bytes> {
            Ok(Bytes::from(vec![0u8; size]))
        }

        async fn delete_file(&self, path: &Path) -> Result<()> {
            log::info!("Mock delete: {}", path.display());
            Ok(())
        }

        fn clone_box(&self) -> Box<dyn Plugin> {
            Box::new(self.clone())
        }
    }

    #[tokio::test]
    async fn test_sync_manager() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut manager = SyncManager::new(temp_dir.path().to_path_buf());

        // Registrar plugin mock
        manager.register_plugin(Box::new(MockPlugin {
            name: "mock".to_string(),
        }));

        // Criar arquivo de teste
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test data").await?;

        // Testar sincronização
        manager.sync_file(&test_file).await?;

        // Testar download
        let download_path = temp_dir.path().join("downloaded.txt");
        manager
            .download_file(
                &manager.plugins[0],
                "test.txt",
                &download_path,
                "test data".len() as u64,
            )
            .await?;

        // Testar deleção
        manager.delete_file(&test_file).await?;
        assert!(!test_file.exists());

        Ok(())
    }
}
