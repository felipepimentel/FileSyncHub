use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::{
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::mpsc;

pub struct FileWatcher {
    path: PathBuf,
    watcher: Option<Debouncer<RecommendedWatcher>>,
}

impl FileWatcher {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            watcher: None,
        }
    }

    pub async fn start<F, Fut>(&mut self, callback: F) -> anyhow::Result<()>
    where
        F: Fn(DebouncedEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel(100);
        let callback = Arc::new(callback);

        let mut debouncer = new_debouncer(
            Duration::from_millis(2000),
            move |events: Result<Vec<DebouncedEvent>, notify::Error>| {
                if let Ok(events) = events {
                    for event in events {
                        if let Err(e) = tx.blocking_send(event) {
                            log::error!("Error sending event: {}", e);
                        }
                    }
                }
            },
        )?;

        debouncer.watcher().watch(&self.path, RecursiveMode::Recursive)?;
        self.watcher = Some(debouncer);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let callback = Arc::clone(&callback);
                callback(event).await;
            }
        });

        Ok(())
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(watcher) = self.watcher.take() {
            drop(watcher);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_file_watcher() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.txt");

        // Create watcher
        let mut watcher = FileWatcher::new(temp_dir.path().to_path_buf());

        // Create channel for test events
        let (tx, mut rx) = mpsc::channel(100);

        // Start watching
        watcher
            .start(move |event| {
                let tx = tx.clone();
                async move {
                    let _ = tx.send(event).await;
                }
            })
            .await?;

        // Create test file
        tokio::fs::write(&test_file, b"test data").await?;

        // Wait for event
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await?
            .expect("No event received");

        assert_eq!(event.path, test_file);

        // Stop watcher
        watcher.stop().await?;

        Ok(())
    }
}
