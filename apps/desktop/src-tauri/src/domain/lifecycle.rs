use super::provider::ProviderId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationLifecycleState {
    NotInstalled,
    Installed,
    Ready,
    Starting,
    Running,
    Stopping,
    Switching,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessOwnership {
    #[serde(rename = "started_by_ai_hub")]
    StartedByAIHub,
    AlreadyRunningExternally,
    DiscoveredRunning,
    OwnershipUnknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub process_name: String,
    pub start_time_ms: Option<u64>,
    pub ownership: ProcessOwnership,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time_ticks: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,
}

impl ProcessIdentity {
    pub fn new(
        pid: u32,
        process_name: String,
        start_time_ms: Option<u64>,
        ownership: ProcessOwnership,
    ) -> Self {
        Self {
            pid,
            process_name,
            start_time_ms,
            ownership,
            start_time_ticks: None,
            executable_path: None,
        }
    }

    pub fn with_validation_details(
        mut self,
        start_time_ticks: Option<u64>,
        executable_path: Option<String>,
    ) -> Self {
        self.start_time_ticks = start_time_ticks;
        self.executable_path = executable_path;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManagedApplication {
    pub provider_id: ProviderId,
    pub lifecycle_state: ApplicationLifecycleState,
    pub process: Option<ProcessIdentity>,
    pub is_active: bool,
    pub last_state_change_ms: u64,
    pub error_message: Option<String>,
}

impl ManagedApplication {
    pub fn new_initial(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            lifecycle_state: ApplicationLifecycleState::Unknown,
            process: None,
            is_active: false,
            last_state_change_ms: 0,
            error_message: None,
        }
    }
}
