use super::provider::{LaunchType, ProviderId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledApplication {
    pub provider_id: ProviderId,
    pub display_name: String,
    pub version: Option<String>,
    pub launch_type: LaunchType,
    pub executable_path: String,
    pub launch_args: Vec<String>,
    pub icon_path: Option<String>,
    pub package_id: Option<String>,
}
