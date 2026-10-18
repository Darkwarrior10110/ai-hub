use super::{
    lifecycle::ProcessIdentity,
    provider::{LaunchType, ProviderId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchStatus {
    Rejected,
    LaunchFailed,
    LaunchRequested,
    Running,
    AlreadyRunning,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchRequest {
    pub provider_id: ProviderId,
    pub launch_type: Option<LaunchType>,
    pub custom_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchResult {
    pub provider_id: ProviderId,
    pub status: LaunchStatus,
    pub process: Option<ProcessIdentity>,
    pub details: Option<String>,
    pub launched_at_ms: u64,
}
