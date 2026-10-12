use super::provider::{LaunchType, ProviderId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationSource {
    Snap,
    DesktopEntry,
    Executable,
    AppImage,
    WindowsApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchConfidence {
    Confirmed,
    Ambiguous,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryStatus {
    Confirmed,
    Ambiguous,
    NotInstalled,
    Unavailable,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredSnapPackage {
    pub name: String,
    pub version: String,
    pub revision: String,
    pub tracking: String,
    pub publisher: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderDiscoveryResult {
    pub provider_id: ProviderId,
    pub status: DiscoveryStatus,
    pub match_confidence: MatchConfidence,
    pub source: Option<ApplicationSource>,
    pub matched_package: Option<String>,
    pub version: Option<String>,
    pub executable_path: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoverySnapshot {
    pub is_snap_available: bool,
    pub scanned_at_ms: u64,
    pub total_packages_scanned: usize,
    pub results: Vec<ProviderDiscoveryResult>,
    pub diagnostics: Option<String>,
}

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
