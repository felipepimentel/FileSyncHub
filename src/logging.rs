use anyhow::Result;
use log::{Level, LevelFilter, Metadata, Record};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::sync::RwLock;

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_LOG_FILES: usize = 5;

pub struct Logger {
    log_dir: PathBuf,
    current_log: Arc<RwLock<File>>,
    level: Level,
}

impl Logger {
    pub fn new(log_dir: PathBuf, level: Level) -> Result<Self> {
        fs::create_dir_all(&log_dir)?;
        let current_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("filesynchub.log"))?;

        Ok(Self {
            log_dir,
            current_log: Arc::new(RwLock::new(current_log)),
            level,
        })
    }

    /// Inicializa o logger
    pub fn init(log_dir: PathBuf, level: Level) -> Result<()> {
        let logger = Self::new(log_dir, level)?;
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Trace);
        Ok(())
    }

    /// Rotaciona os arquivos de log se necessário
    async fn rotate_logs(&self) -> Result<()> {
        let current_log = self.log_dir.join("filesynchub.log");
        let metadata = fs::metadata(&current_log)?;

        if metadata.len() > MAX_LOG_SIZE {
            // Renomear logs existentes
            for i in (1..MAX_LOG_FILES).rev() {
                let old_path = self.log_dir.join(format!("filesynchub.{}.log", i));
                let new_path = self.log_dir.join(format!("filesynchub.{}.log", i + 1));
                if old_path.exists() {
                    if i == MAX_LOG_FILES - 1 {
                        fs::remove_file(&old_path)?;
                    } else {
                        fs::rename(&old_path, &new_path)?;
                    }
                }
            }

            // Renomear log atual
            fs::rename(&current_log, self.log_dir.join("filesynchub.1.log"))?;

            // Criar novo arquivo de log
            let new_log = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&current_log)?;

            let mut current = self.current_log.write().await;
            *current = new_log;
        }

        Ok(())
    }

    /// Limpa logs antigos
    pub async fn cleanup_old_logs(&self) -> Result<()> {
        let entries = fs::read_dir(&self.log_dir)?;
        let mut log_files: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("filesynchub")
            })
            .collect();

        // Ordenar por data de modificação (mais recente primeiro)
        log_files
            .sort_by_key(|entry| std::cmp::Reverse(entry.metadata().unwrap().modified().unwrap()));

        // Remover logs excedentes
        for entry in log_files.iter().skip(MAX_LOG_FILES) {
            fs::remove_file(entry.path())?;
        }

        Ok(())
    }

    /// Formata uma mensagem de log
    fn format_log(&self, record: &Record) -> String {
        let timestamp = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| String::from("timestamp error"));

        format!(
            "[{} {} {}:{}] {}\n",
            timestamp,
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = self.format_log(record);
            let current_log = self.current_log.clone();

            tokio::spawn(async move {
                if let Ok(mut file) = current_log.write().await {
                    let _ = file.write_all(message.as_bytes());
                    let _ = file.flush();
                }
            });
        }
    }

    fn flush(&self) {
        let current_log = self.current_log.clone();
        tokio::spawn(async move {
            if let Ok(mut file) = current_log.write().await {
                let _ = file.flush();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_logger() -> Result<()> {
        let temp_dir = tempdir()?;
        let logger = Logger::new(temp_dir.path().to_path_buf(), Level::Debug)?;

        // Testar rotação de logs
        let message = "Test log message\n".repeat(1000000); // Criar mensagem grande
        let mut file = logger.current_log.write().await;
        file.write_all(message.as_bytes())?;
        drop(file);

        logger.rotate_logs().await?;

        // Verificar se os arquivos foram criados
        assert!(temp_dir.path().join("filesynchub.log").exists());
        assert!(temp_dir.path().join("filesynchub.1.log").exists());

        // Testar limpeza de logs
        logger.cleanup_old_logs().await?;

        Ok(())
    }

    #[test]
    fn test_log_format() {
        let temp_dir = tempdir().unwrap();
        let logger = Logger::new(temp_dir.path().to_path_buf(), Level::Debug).unwrap();

        let record = Record::builder()
            .args(format_args!("test message"))
            .level(Level::Info)
            .target("test")
            .file(Some("test.rs"))
            .line(Some(42))
            .build();

        let formatted = logger.format_log(&record);
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test.rs:42"));
        assert!(formatted.contains("test message"));
    }
}
