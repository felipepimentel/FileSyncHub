use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    fs,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

const CACHE_VERSION: u32 = 1;
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hora
const MAX_MEMORY_CACHE_SIZE: usize = 100 * 1024 * 1024; // 100MB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub hash: String,
    pub size: u64,
    pub modified: SystemTime,
    pub plugin_metadata: HashMap<String, String>,
    pub last_sync: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub version: u32,
    pub created: SystemTime,
    pub last_cleanup: SystemTime,
}

pub struct Cache {
    cache_dir: PathBuf,
    metadata: Arc<RwLock<CacheMetadata>>,
    entries: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    memory_cache: Arc<RwLock<lru::LruCache<String, Bytes>>>,
    ttl: Duration,
}

impl Cache {
    pub async fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir).await?;
        let metadata_path = cache_dir.join("metadata.json");

        let metadata = if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&content)?
        } else {
            CacheMetadata {
                version: CACHE_VERSION,
                created: SystemTime::now(),
                last_cleanup: SystemTime::now(),
            }
        };

        let entries = Self::load_entries(&cache_dir).await?;

        Ok(Self {
            cache_dir,
            metadata: Arc::new(RwLock::new(metadata)),
            entries: Arc::new(RwLock::new(entries)),
            memory_cache: Arc::new(RwLock::new(lru::LruCache::new(MAX_MEMORY_CACHE_SIZE))),
            ttl: DEFAULT_CACHE_TTL,
        })
    }

    /// Carrega as entradas do cache do disco
    async fn load_entries(cache_dir: &Path) -> Result<HashMap<PathBuf, CacheEntry>> {
        let entries_path = cache_dir.join("entries.json");
        if entries_path.exists() {
            let content = fs::read_to_string(&entries_path).await?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(HashMap::new())
        }
    }

    /// Salva as entradas do cache no disco
    async fn save_entries(&self) -> Result<()> {
        let entries = self.entries.read().await;
        let content = serde_json::to_string_pretty(&*entries)?;
        fs::write(self.cache_dir.join("entries.json"), content).await?;
        Ok(())
    }

    /// Salva os metadados do cache no disco
    async fn save_metadata(&self) -> Result<()> {
        let metadata = self.metadata.read().await;
        let content = serde_json::to_string_pretty(&*metadata)?;
        fs::write(self.cache_dir.join("metadata.json"), content).await?;
        Ok(())
    }

    /// Obtém uma entrada do cache
    pub async fn get_entry(&self, path: &Path) -> Option<CacheEntry> {
        let entries = self.entries.read().await;
        entries.get(path).cloned()
    }

    /// Atualiza ou insere uma entrada no cache
    pub async fn set_entry(&self, path: PathBuf, entry: CacheEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.insert(path, entry);
        self.save_entries().await?;
        Ok(())
    }

    /// Remove uma entrada do cache
    pub async fn remove_entry(&self, path: &Path) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.remove(path);
        self.save_entries().await?;
        Ok(())
    }

    /// Obtém dados do cache em memória
    pub async fn get_memory(&self, key: &str) -> Option<Bytes> {
        let cache = self.memory_cache.read().await;
        cache.get(key).cloned()
    }

    /// Armazena dados no cache em memória
    pub async fn set_memory(&self, key: String, data: Bytes) {
        let mut cache = self.memory_cache.write().await;
        cache.put(key, data);
    }

    /// Limpa entradas expiradas do cache
    pub async fn cleanup(&self) -> Result<()> {
        let now = SystemTime::now();
        let mut entries = self.entries.write().await;
        let mut metadata = self.metadata.write().await;

        // Remover entradas expiradas
        entries.retain(|_, entry| {
            entry
                .last_sync
                .elapsed()
                .map(|elapsed| elapsed < self.ttl)
                .unwrap_or(false)
        });

        metadata.last_cleanup = now;

        // Salvar alterações
        self.save_entries().await?;
        self.save_metadata().await?;

        Ok(())
    }

    /// Define o TTL (time-to-live) do cache
    pub fn set_ttl(&mut self, ttl: Duration) {
        self.ttl = ttl;
    }

    /// Obtém um lock de leitura para as entradas do cache
    pub async fn read_entries(&self) -> RwLockReadGuard<'_, HashMap<PathBuf, CacheEntry>> {
        self.entries.read().await
    }

    /// Obtém um lock de escrita para as entradas do cache
    pub async fn write_entries(&self) -> RwLockWriteGuard<'_, HashMap<PathBuf, CacheEntry>> {
        self.entries.write().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cache_operations() -> Result<()> {
        let temp_dir = tempdir()?;
        let cache = Cache::new(temp_dir.path().to_path_buf()).await?;

        // Testar inserção
        let entry = CacheEntry {
            hash: "test_hash".to_string(),
            size: 1000,
            modified: SystemTime::now(),
            plugin_metadata: HashMap::new(),
            last_sync: SystemTime::now(),
        };

        let path = PathBuf::from("test.txt");
        cache.set_entry(path.clone(), entry.clone()).await?;

        // Testar recuperação
        let retrieved = cache.get_entry(&path).await.unwrap();
        assert_eq!(retrieved.hash, entry.hash);

        // Testar remoção
        cache.remove_entry(&path).await?;
        assert!(cache.get_entry(&path).await.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_cache() -> Result<()> {
        let temp_dir = tempdir()?;
        let cache = Cache::new(temp_dir.path().to_path_buf()).await?;

        // Testar cache em memória
        let key = "test_key".to_string();
        let data = Bytes::from("test data");
        cache.set_memory(key.clone(), data.clone()).await;

        let retrieved = cache.get_memory(&key).await.unwrap();
        assert_eq!(retrieved, data);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_cleanup() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut cache = Cache::new(temp_dir.path().to_path_buf()).await?;
        cache.set_ttl(Duration::from_secs(0)); // Definir TTL para 0 para teste

        // Inserir entrada
        let entry = CacheEntry {
            hash: "test_hash".to_string(),
            size: 1000,
            modified: SystemTime::now(),
            plugin_metadata: HashMap::new(),
            last_sync: SystemTime::now(),
        };

        let path = PathBuf::from("test.txt");
        cache.set_entry(path.clone(), entry).await?;

        // Executar limpeza
        tokio::time::sleep(Duration::from_millis(10)).await;
        cache.cleanup().await?;

        // Verificar se a entrada foi removida
        assert!(cache.get_entry(&path).await.is_none());

        Ok(())
    }
}
