use anyhow::Result;
use bytes::Bytes;
use futures::stream::{self, StreamExt};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, RwLock, Semaphore},
};

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
const MAX_CONCURRENT_UPLOADS: usize = 3;
const CACHE_SIZE: usize = 100;

#[derive(Clone)]
pub struct ChunkMetadata {
    pub hash: String,
    pub size: usize,
    pub offset: u64,
}

pub struct PerformanceOptimizer {
    chunk_cache: Arc<RwLock<lru::LruCache<String, Bytes>>>,
    upload_semaphore: Arc<Semaphore>,
    chunk_metadata: Arc<RwLock<HashMap<PathBuf, Vec<ChunkMetadata>>>>,
    temp_dir: PathBuf,
}

impl PerformanceOptimizer {
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            chunk_cache: Arc::new(RwLock::new(lru::LruCache::new(CACHE_SIZE))),
            upload_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS)),
            chunk_metadata: Arc::new(RwLock::new(HashMap::new())),
            temp_dir,
        }
    }

    /// Processa um arquivo em chunks para upload otimizado
    pub async fn process_file(&self, path: &Path) -> Result<Vec<ChunkMetadata>> {
        let file = File::open(path).await?;
        let file_size = file.metadata().await?.len();
        let mut chunks = Vec::new();
        let mut offset = 0;

        // Dividir arquivo em chunks
        while offset < file_size {
            let chunk_size = std::cmp::min(CHUNK_SIZE as u64, file_size - offset);
            let mut buffer = vec![0; chunk_size as usize];

            let mut file = File::open(path).await?;
            file.seek(std::io::SeekFrom::Start(offset)).await?;
            file.read_exact(&mut buffer).await?;

            let hash = {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                format!("{:x}", hasher.finalize())
            };

            // Armazenar chunk no cache se não existir
            let mut cache = self.chunk_cache.write().await;
            if !cache.contains(&hash) {
                cache.put(hash.clone(), Bytes::from(buffer.clone()));
            }

            chunks.push(ChunkMetadata {
                hash,
                size: chunk_size as usize,
                offset,
            });

            offset += chunk_size;
        }

        // Armazenar metadados dos chunks
        self.chunk_metadata
            .write()
            .await
            .insert(path.to_path_buf(), chunks.clone());

        Ok(chunks)
    }

    /// Upload paralelo de chunks
    pub async fn upload_chunks<F>(&self, path: &Path, upload_fn: F) -> Result<()>
    where
        F: Fn(Bytes, u64) -> futures::future::BoxFuture<'static, Result<()>>
            + Send
            + Sync
            + 'static,
    {
        let chunks = if let Some(chunks) = self.chunk_metadata.read().await.get(path) {
            chunks.clone()
        } else {
            self.process_file(path).await?
        };

        let upload_fn = Arc::new(upload_fn);
        let mut tasks = Vec::new();

        // Criar tasks para upload paralelo
        for chunk in chunks {
            let cache = self.chunk_cache.clone();
            let semaphore = self.upload_semaphore.clone();
            let upload_fn = upload_fn.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;

                // Obter chunk do cache
                let data = {
                    let cache = cache.read().await;
                    if let Some(data) = cache.get(&chunk.hash) {
                        data.clone()
                    } else {
                        return Err(anyhow::anyhow!("Chunk not found in cache"));
                    }
                };

                // Upload do chunk
                upload_fn(data, chunk.offset).await?;

                Ok::<_, anyhow::Error>(())
            });

            tasks.push(task);
        }

        // Aguardar conclusão de todos os uploads
        futures::future::try_join_all(tasks).await?;

        Ok(())
    }

    /// Download paralelo de chunks
    pub async fn download_chunks<F>(
        &self,
        path: &Path,
        total_size: u64,
        download_fn: F,
    ) -> Result<()>
    where
        F: Fn(u64, usize) -> futures::future::BoxFuture<'static, Result<Bytes>>
            + Send
            + Sync
            + 'static,
    {
        let mut offset = 0;
        let mut tasks = Vec::new();
        let temp_file = Arc::new(Mutex::new(
            File::create(self.temp_dir.join(path.file_name().unwrap())).await?,
        ));

        // Criar tasks para download paralelo
        while offset < total_size {
            let chunk_size = std::cmp::min(CHUNK_SIZE as u64, total_size - offset);
            let download_fn = Arc::new(download_fn);
            let temp_file = temp_file.clone();
            let current_offset = offset;

            let task = tokio::spawn(async move {
                let data = download_fn(current_offset, chunk_size as usize).await?;

                // Escrever chunk no arquivo temporário
                let mut file = temp_file.lock().await;
                file.seek(std::io::SeekFrom::Start(current_offset)).await?;
                file.write_all(&data).await?;

                Ok::<_, anyhow::Error>(())
            });

            tasks.push(task);
            offset += chunk_size;
        }

        // Aguardar conclusão de todos os downloads
        futures::future::try_join_all(tasks).await?;

        // Mover arquivo temporário para localização final
        tokio::fs::rename(self.temp_dir.join(path.file_name().unwrap()), path).await?;

        Ok(())
    }

    /// Limpa o cache de chunks
    pub async fn clear_cache(&self) {
        self.chunk_cache.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_chunk_processing() -> Result<()> {
        let temp_dir = tempdir()?;
        let optimizer = PerformanceOptimizer::new(temp_dir.path().to_path_buf());

        // Criar arquivo de teste
        let test_file = temp_dir.path().join("test.txt");
        let test_data = vec![0u8; CHUNK_SIZE * 2]; // 2MB de dados
        tokio::fs::write(&test_file, &test_data).await?;

        // Processar arquivo em chunks
        let chunks = optimizer.process_file(&test_file).await?;

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].size, CHUNK_SIZE);
        assert_eq!(chunks[1].size, CHUNK_SIZE);

        Ok(())
    }

    #[tokio::test]
    async fn test_parallel_upload() -> Result<()> {
        let temp_dir = tempdir()?;
        let optimizer = PerformanceOptimizer::new(temp_dir.path().to_path_buf());

        // Criar arquivo de teste
        let test_file = temp_dir.path().join("test.txt");
        let test_data = vec![0u8; CHUNK_SIZE * 3]; // 3MB de dados
        tokio::fs::write(&test_file, &test_data).await?;

        // Simular upload
        let uploaded_chunks = Arc::new(Mutex::new(Vec::new()));
        let uploaded_chunks_clone = uploaded_chunks.clone();

        optimizer
            .upload_chunks(&test_file, move |data, offset| {
                let uploaded_chunks = uploaded_chunks_clone.clone();
                Box::pin(async move {
                    uploaded_chunks.lock().await.push((offset, data.len()));
                    Ok(())
                })
            })
            .await?;

        let final_chunks = uploaded_chunks.lock().await;
        assert_eq!(final_chunks.len(), 3);

        Ok(())
    }
}
