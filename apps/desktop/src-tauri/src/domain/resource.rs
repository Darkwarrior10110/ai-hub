use super::provider::ProviderId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessMetric {
    pub pid: u32,
    pub cpu_percentage: f32,
    pub memory_rss_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemMemory {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub used_percentage: u8,
    pub swap_total_bytes: u64,
    pub swap_free_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ResourceSnapshot {
    pub provider_id: Option<ProviderId>,
    pub total_cpu_percentage: Option<f32>,
    pub total_memory_bytes: Option<u64>,
    pub system_memory: Option<SystemMemory>,
    pub memory_pressure_warning: bool,
    pub memory_pressure_message: Option<String>,
    pub tracked_processes_count: usize,
    pub process_metrics: Vec<ProcessMetric>,
    pub is_available: bool,
    pub diagnostics: Option<String>,
    pub timestamp_ms: u64,
}
