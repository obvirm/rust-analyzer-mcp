use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct Metrics {
    request_counts: Arc<RwLock<HashMap<String, u64>>>,
    request_latencies: Arc<RwLock<Vec<Duration>>>,
    lsp_response_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    error_counts: Arc<RwLock<HashMap<String, u64>>>,
    started_at: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            request_counts: Arc::new(RwLock::new(HashMap::new())),
            request_latencies: Arc::new(RwLock::new(Vec::new())),
            lsp_response_times: Arc::new(RwLock::new(HashMap::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            started_at: Instant::now(),
        }
    }

    pub async fn record_request(&self, tool_name: &str, duration: Duration) {
        {
            let mut counts = self.request_counts.write().await;
            *counts.entry(tool_name.to_string()).or_insert(0) += 1;
        }
        {
            let mut latencies = self.request_latencies.write().await;
            latencies.push(duration);
            if latencies.len() > 1000 {
                latencies.remove(0);
            }
        }
    }

    pub async fn record_lsp_call(&self, method: &str, duration: Duration) {
        let mut times = self.lsp_response_times.write().await;
        times
            .entry(method.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }

    pub async fn record_error(&self, error_type: &str) {
        let mut counts = self.error_counts.write().await;
        *counts.entry(error_type.to_string()).or_insert(0) += 1;
    }

    pub async fn get_summary(&self) -> MetricsSummary {
        let request_counts = self.request_counts.read().await.clone();
        let error_counts = self.error_counts.read().await.clone();

        let latencies = self.request_latencies.read().await;
        let avg_latency = if latencies.is_empty() {
            Duration::ZERO
        } else {
            latencies.iter().sum::<Duration>() / latencies.len() as u32
        };

        let p99_latency = if latencies.is_empty() {
            Duration::ZERO
        } else {
            let mut sorted = latencies.clone();
            sorted.sort();
            let idx = ((sorted.len() as f64) * 0.99) as usize;
            sorted[idx.min(sorted.len() - 1)]
        };

        MetricsSummary {
            uptime: self.started_at.elapsed(),
            total_requests: request_counts.values().sum(),
            requests_by_tool: request_counts,
            errors_by_type: error_counts,
            avg_request_latency: avg_latency,
            p99_request_latency: p99_latency,
            lsp_stats: self.lsp_response_times.read().await.clone(),
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSummary {
    pub uptime: Duration,
    pub total_requests: u64,
    pub requests_by_tool: HashMap<String, u64>,
    pub errors_by_type: HashMap<String, u64>,
    pub avg_request_latency: Duration,
    pub p99_request_latency: Duration,
    pub lsp_stats: HashMap<String, Vec<Duration>>,
}
