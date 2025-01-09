use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    pub path: PathBuf,
    pub size: u64,
    pub sync_count: u64,
    pub last_sync: SystemTime,
    pub total_sync_time: Duration,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    pub name: String,
    pub files_synced: u64,
    pub bytes_transferred: u64,
    pub total_time: Duration,
    pub error_count: u64,
    pub last_sync: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub start_time: SystemTime,
    pub total_files: u64,
    pub total_bytes: u64,
    pub total_sync_time: Duration,
    pub total_errors: u64,
    pub last_cleanup: SystemTime,
}

pub struct StatsCollector {
    file_stats: Arc<RwLock<HashMap<PathBuf, FileStats>>>,
    plugin_stats: Arc<RwLock<HashMap<String, PluginStats>>>,
    system_stats: Arc<RwLock<SystemStats>>,
    stats_dir: PathBuf,
}

impl StatsCollector {
    pub async fn new(stats_dir: PathBuf) -> Result<Self> {
        tokio::fs::create_dir_all(&stats_dir).await?;

        let system_stats = SystemStats {
            start_time: SystemTime::now(),
            total_files: 0,
            total_bytes: 0,
            total_sync_time: Duration::default(),
            total_errors: 0,
            last_cleanup: SystemTime::now(),
        };

        Ok(Self {
            file_stats: Arc::new(RwLock::new(HashMap::new())),
            plugin_stats: Arc::new(RwLock::new(HashMap::new())),
            system_stats: Arc::new(RwLock::new(system_stats)),
            stats_dir,
        })
    }

    /// Registra uma operação de sincronização de arquivo
    pub async fn record_file_sync(
        &self,
        path: PathBuf,
        size: u64,
        duration: Duration,
        success: bool,
    ) {
        let mut file_stats = self.file_stats.write().await;
        let stats = file_stats.entry(path.clone()).or_insert_with(|| FileStats {
            path,
            size,
            sync_count: 0,
            last_sync: SystemTime::now(),
            total_sync_time: Duration::default(),
            error_count: 0,
        });

        stats.sync_count += 1;
        stats.last_sync = SystemTime::now();
        stats.total_sync_time += duration;
        if !success {
            stats.error_count += 1;
        }

        // Atualizar estatísticas do sistema
        let mut system = self.system_stats.write().await;
        system.total_files += 1;
        system.total_bytes += size;
        system.total_sync_time += duration;
        if !success {
            system.total_errors += 1;
        }
    }

    /// Registra estatísticas de plugin
    pub async fn record_plugin_sync(
        &self,
        name: String,
        files: u64,
        bytes: u64,
        duration: Duration,
        errors: u64,
    ) {
        let mut plugin_stats = self.plugin_stats.write().await;
        let stats = plugin_stats
            .entry(name.clone())
            .or_insert_with(|| PluginStats {
                name,
                files_synced: 0,
                bytes_transferred: 0,
                total_time: Duration::default(),
                error_count: 0,
                last_sync: SystemTime::now(),
            });

        stats.files_synced += files;
        stats.bytes_transferred += bytes;
        stats.total_time += duration;
        stats.error_count += errors;
        stats.last_sync = SystemTime::now();
    }

    /// Obtém estatísticas de um arquivo específico
    pub async fn get_file_stats(&self, path: &PathBuf) -> Option<FileStats> {
        let stats = self.file_stats.read().await;
        stats.get(path).cloned()
    }

    /// Obtém estatísticas de um plugin específico
    pub async fn get_plugin_stats(&self, name: &str) -> Option<PluginStats> {
        let stats = self.plugin_stats.read().await;
        stats.get(name).cloned()
    }

    /// Obtém estatísticas do sistema
    pub async fn get_system_stats(&self) -> SystemStats {
        self.system_stats.read().await.clone()
    }

    /// Salva as estatísticas em disco
    pub async fn save_stats(&self) -> Result<()> {
        // Salvar estatísticas de arquivos
        let file_stats = self.file_stats.read().await;
        let content = serde_json::to_string_pretty(&*file_stats)?;
        tokio::fs::write(self.stats_dir.join("file_stats.json"), content).await?;

        // Salvar estatísticas de plugins
        let plugin_stats = self.plugin_stats.read().await;
        let content = serde_json::to_string_pretty(&*plugin_stats)?;
        tokio::fs::write(self.stats_dir.join("plugin_stats.json"), content).await?;

        // Salvar estatísticas do sistema
        let system_stats = self.system_stats.read().await;
        let content = serde_json::to_string_pretty(&*system_stats)?;
        tokio::fs::write(self.stats_dir.join("system_stats.json"), content).await?;

        Ok(())
    }

    /// Carrega as estatísticas do disco
    pub async fn load_stats(&self) -> Result<()> {
        // Carregar estatísticas de arquivos
        let path = self.stats_dir.join("file_stats.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let stats: HashMap<PathBuf, FileStats> = serde_json::from_str(&content)?;
            *self.file_stats.write().await = stats;
        }

        // Carregar estatísticas de plugins
        let path = self.stats_dir.join("plugin_stats.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let stats: HashMap<String, PluginStats> = serde_json::from_str(&content)?;
            *self.plugin_stats.write().await = stats;
        }

        // Carregar estatísticas do sistema
        let path = self.stats_dir.join("system_stats.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let stats: SystemStats = serde_json::from_str(&content)?;
            *self.system_stats.write().await = stats;
        }

        Ok(())
    }

    /// Limpa estatísticas antigas
    pub async fn cleanup_old_stats(&self, max_age: Duration) -> Result<()> {
        let now = SystemTime::now();

        // Limpar estatísticas de arquivos antigas
        let mut file_stats = self.file_stats.write().await;
        file_stats.retain(|_, stats| {
            stats
                .last_sync
                .elapsed()
                .map(|age| age < max_age)
                .unwrap_or(false)
        });

        // Limpar estatísticas de plugins antigas
        let mut plugin_stats = self.plugin_stats.write().await;
        plugin_stats.retain(|_, stats| {
            stats
                .last_sync
                .elapsed()
                .map(|age| age < max_age)
                .unwrap_or(false)
        });

        // Atualizar timestamp de limpeza
        let mut system = self.system_stats.write().await;
        system.last_cleanup = now;

        // Salvar alterações
        self.save_stats().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_stats_collection() -> Result<()> {
        let temp_dir = tempdir()?;
        let collector = StatsCollector::new(temp_dir.path().to_path_buf()).await?;

        // Testar registro de sincronização de arquivo
        let path = PathBuf::from("test.txt");
        collector
            .record_file_sync(path.clone(), 1000, Duration::from_secs(1), true)
            .await;

        // Verificar estatísticas do arquivo
        let stats = collector.get_file_stats(&path).await.unwrap();
        assert_eq!(stats.size, 1000);
        assert_eq!(stats.sync_count, 1);
        assert_eq!(stats.error_count, 0);

        // Testar registro de estatísticas de plugin
        collector
            .record_plugin_sync(
                "test_plugin".to_string(),
                1,
                1000,
                Duration::from_secs(1),
                0,
            )
            .await;

        // Verificar estatísticas do plugin
        let stats = collector.get_plugin_stats("test_plugin").await.unwrap();
        assert_eq!(stats.files_synced, 1);
        assert_eq!(stats.bytes_transferred, 1000);
        assert_eq!(stats.error_count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_stats_persistence() -> Result<()> {
        let temp_dir = tempdir()?;
        let collector = StatsCollector::new(temp_dir.path().to_path_buf()).await?;

        // Registrar algumas estatísticas
        collector
            .record_file_sync(
                PathBuf::from("test.txt"),
                1000,
                Duration::from_secs(1),
                true,
            )
            .await;

        // Salvar estatísticas
        collector.save_stats().await?;

        // Criar novo coletor e carregar estatísticas
        let new_collector = StatsCollector::new(temp_dir.path().to_path_buf()).await?;
        new_collector.load_stats().await?;

        // Verificar se as estatísticas foram carregadas corretamente
        let stats = new_collector
            .get_file_stats(&PathBuf::from("test.txt"))
            .await
            .unwrap();
        assert_eq!(stats.size, 1000);
        assert_eq!(stats.sync_count, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_stats_cleanup() -> Result<()> {
        let temp_dir = tempdir()?;
        let collector = StatsCollector::new(temp_dir.path().to_path_buf()).await?;

        // Registrar estatísticas
        collector
            .record_file_sync(
                PathBuf::from("test.txt"),
                1000,
                Duration::from_secs(1),
                true,
            )
            .await;

        // Limpar estatísticas antigas (usando TTL de 0 para teste)
        tokio::time::sleep(Duration::from_millis(10)).await;
        collector.cleanup_old_stats(Duration::from_secs(0)).await?;

        // Verificar se as estatísticas foram removidas
        assert!(collector
            .get_file_stats(&PathBuf::from("test.txt"))
            .await
            .is_none());

        Ok(())
    }
}
