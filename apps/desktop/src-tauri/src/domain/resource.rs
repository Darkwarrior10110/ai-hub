use super::provider::ProviderId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessMetric {
    pub pid: u32,
    pub cpu_percentage: f32,
    pub memory_rss_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceSnapshot {
    pub provider_id: Option<ProviderId>,
    pub total_cpu_percentage: Option<f32>,
    pub total_memory_bytes: Option<u64>,
    pub tracked_processes_count: usize,
    pub process_metrics: Vec<ProcessMetric>,
    pub is_available: bool,
    pub timestamp_ms: u64,
}

impl Default for ResourceSnapshot {
    fn default() -> Self {
        Self {
            provider_id: None,
            total_cpu_percentage: None,
            total_memory_bytes: None,
            tracked_processes_count: 0,
            process_metrics: Vec::new(),
            is_available: false,
            timestamp_ms: 0,
        }
    }
}
