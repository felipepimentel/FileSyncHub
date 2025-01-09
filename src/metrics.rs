use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SyncMetrics {
    pub total_bytes: u64,
    pub files_processed: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct PluginMetrics {
    pub name: String,
    pub uploads: u64,
    pub downloads: u64,
    pub errors: u64,
    pub total_bytes: u64,
    pub average_speed: f64,
}

#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub operation: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub bytes_processed: u64,
    pub success: bool,
}

pub struct MetricsCollector {
    sync_metrics: Arc<RwLock<SyncMetrics>>,
    plugin_metrics: Arc<RwLock<HashMap<String, PluginMetrics>>>,
    operation_history: Arc<RwLock<Vec<OperationMetrics>>>,
}

impl Default for SyncMetrics {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            files_processed: 0,
            success_count: 0,
            error_count: 0,
            duration: Duration::default(),
        }
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            sync_metrics: Arc::new(RwLock::new(SyncMetrics::default())),
            plugin_metrics: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Registra o início de uma operação
    pub async fn start_operation(&self, operation: String) -> OperationMetrics {
        let metrics = OperationMetrics {
            operation,
            start_time: Instant::now(),
            end_time: None,
            bytes_processed: 0,
            success: false,
        };

        self.operation_history.write().await.push(metrics.clone());
        metrics
    }

    /// Atualiza uma operação em andamento
    pub async fn update_operation(&self, metrics: &mut OperationMetrics, bytes: u64) {
        metrics.bytes_processed += bytes;

        let mut history = self.operation_history.write().await;
        if let Some(last) = history.last_mut() {
            if last.operation == metrics.operation {
                last.bytes_processed = metrics.bytes_processed;
            }
        }
    }

    /// Finaliza uma operação
    pub async fn finish_operation(&self, metrics: &mut OperationMetrics, success: bool) {
        metrics.end_time = Some(Instant::now());
        metrics.success = success;

        let mut history = self.operation_history.write().await;
        if let Some(last) = history.last_mut() {
            if last.operation == metrics.operation {
                last.end_time = metrics.end_time;
                last.success = success;
            }
        }

        // Atualizar métricas de sincronização
        let mut sync_metrics = self.sync_metrics.write().await;
        sync_metrics.total_bytes += metrics.bytes_processed;
        sync_metrics.files_processed += 1;
        if success {
            sync_metrics.success_count += 1;
        } else {
            sync_metrics.error_count += 1;
        }
        sync_metrics.duration += metrics.end_time.unwrap().duration_since(metrics.start_time);
    }

    /// Registra métricas de plugin
    pub async fn record_plugin_metrics(
        &self,
        name: String,
        operation: &str,
        bytes: u64,
        success: bool,
    ) {
        let mut plugin_metrics = self.plugin_metrics.write().await;
        let metrics = plugin_metrics
            .entry(name.clone())
            .or_insert_with(|| PluginMetrics {
                name,
                uploads: 0,
                downloads: 0,
                errors: 0,
                total_bytes: 0,
                average_speed: 0.0,
            });

        match operation {
            "upload" => metrics.uploads += 1,
            "download" => metrics.downloads += 1,
            _ => {}
        }

        if !success {
            metrics.errors += 1;
        }

        metrics.total_bytes += bytes;
    }

    /// Obtém um resumo das métricas
    pub async fn get_summary(&self) -> (SyncMetrics, Vec<PluginMetrics>) {
        let sync_metrics = self.sync_metrics.read().await.clone();
        let plugin_metrics: Vec<_> = self.plugin_metrics.read().await.values().cloned().collect();
        (sync_metrics, plugin_metrics)
    }

    /// Calcula a velocidade média de transferência (bytes/segundo)
    pub fn calculate_average_speed(&self, bytes: u64, duration: Duration) -> f64 {
        if duration.as_secs_f64() > 0.0 {
            bytes as f64 / duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Limpa o histórico de operações
    pub async fn clear_history(&self) {
        self.operation_history.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        // Testar registro de operação
        let mut op = collector.start_operation("upload".to_string()).await;
        sleep(Duration::from_millis(100)).await;
        collector.update_operation(&mut op, 1000).await;
        collector.finish_operation(&mut op, true).await;

        // Verificar métricas
        let (sync_metrics, _) = collector.get_summary().await;
        assert_eq!(sync_metrics.files_processed, 1);
        assert_eq!(sync_metrics.total_bytes, 1000);
        assert_eq!(sync_metrics.success_count, 1);
    }

    #[tokio::test]
    async fn test_plugin_metrics() {
        let collector = MetricsCollector::new();

        // Registrar métricas de plugin
        collector
            .record_plugin_metrics("google_drive".to_string(), "upload", 1000, true)
            .await;
        collector
            .record_plugin_metrics("google_drive".to_string(), "download", 500, true)
            .await;
        collector
            .record_plugin_metrics("google_drive".to_string(), "upload", 0, false)
            .await;

        // Verificar métricas
        let (_, plugin_metrics) = collector.get_summary().await;
        let google_drive = plugin_metrics
            .iter()
            .find(|m| m.name == "google_drive")
            .unwrap();

        assert_eq!(google_drive.uploads, 2);
        assert_eq!(google_drive.downloads, 1);
        assert_eq!(google_drive.errors, 1);
        assert_eq!(google_drive.total_bytes, 1500);
    }

    #[test]
    fn test_speed_calculation() {
        let collector = MetricsCollector::new();
        let speed = collector.calculate_average_speed(1000, Duration::from_secs(2));
        assert_eq!(speed, 500.0);
    }
}
