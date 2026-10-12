use super::{
    snap::{is_snap_available, match_provider_against_snaps, query_installed_snaps},
    traits::RuntimePlatform,
};
use crate::domain::{
    discovery::{
        DiscoverySnapshot, DiscoveryStatus, InstalledApplication, MatchConfidence,
        ProviderDiscoveryResult,
    },
    error::RuntimeError,
    lifecycle::ProcessIdentity,
    provider::{LaunchType, ProviderDefinition},
};
use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Default)]
pub struct LinuxRuntime;

impl LinuxRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl RuntimePlatform for LinuxRuntime {
    fn name(&self) -> &'static str {
        "linux"
    }

    fn discover_all_applications(
        &self,
        providers: &[ProviderDefinition],
    ) -> Result<DiscoverySnapshot, RuntimeError> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if !is_snap_available() {
            let results = providers
                .iter()
                .map(|p| ProviderDiscoveryResult {
                    provider_id: p.id,
                    status: DiscoveryStatus::Unavailable,
                    match_confidence: MatchConfidence::None,
                    source: None,
                    matched_package: None,
                    version: None,
                    executable_path: None,
                    details: Some(
                        "Snap command-line tooling is not installed or available on this system"
                            .to_string(),
                    ),
                })
                .collect();

            return Ok(DiscoverySnapshot {
                is_snap_available: false,
                scanned_at_ms: now_ms,
                total_packages_scanned: 0,
                results,
                diagnostics: Some("Snap tooling unavailable in PATH".to_string()),
            });
        }

        // Query Snap metadata with bounded 5-second timeout
        match query_installed_snaps(Duration::from_secs(5)) {
            Ok(packages) => {
                let results = providers
                    .iter()
                    .map(|p| match_provider_against_snaps(p, &packages))
                    .collect();

                Ok(DiscoverySnapshot {
                    is_snap_available: true,
                    scanned_at_ms: now_ms,
                    total_packages_scanned: packages.len(),
                    results,
                    diagnostics: None,
                })
            }
            Err(e) => {
                let error_desc = e.to_string();
                let results = providers
                    .iter()
                    .map(|p| ProviderDiscoveryResult {
                        provider_id: p.id,
                        status: DiscoveryStatus::Failed,
                        match_confidence: MatchConfidence::None,
                        source: None,
                        matched_package: None,
                        version: None,
                        executable_path: None,
                        details: Some(format!("Snap discovery query failed: {error_desc}")),
                    })
                    .collect();

                Ok(DiscoverySnapshot {
                    is_snap_available: true,
                    scanned_at_ms: now_ms,
                    total_packages_scanned: 0,
                    results,
                    diagnostics: Some(error_desc),
                })
            }
        }
    }

    fn detect_application(
        &self,
        provider: &ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        // First, check Snap metadata if snap is available
        if is_snap_available() {
            if let Ok(packages) = query_installed_snaps(Duration::from_secs(5)) {
                let match_result = match_provider_against_snaps(provider, &packages);
                if match_result.status == DiscoveryStatus::Confirmed {
                    if let Some(pkg_name) = match_result.matched_package {
                        let exe_path = match_result
                            .executable_path
                            .unwrap_or_else(|| format!("/snap/bin/{pkg_name}"));

                        return Ok(Some(InstalledApplication {
                            provider_id: provider.id,
                            display_name: provider.name.clone(),
                            version: match_result.version,
                            launch_type: LaunchType::Snap,
                            executable_path: exe_path,
                            launch_args: Vec::new(),
                            icon_path: None,
                            package_id: Some(pkg_name),
                        }));
                    }
                }
            }
        }

        // Fallback: check candidate executables directly
        for exe in &provider.candidate_executables {
            if Path::new(exe).exists() {
                return Ok(Some(InstalledApplication {
                    provider_id: provider.id,
                    display_name: provider.name.clone(),
                    version: None,
                    launch_type: if exe.starts_with("/snap/") {
                        LaunchType::Snap
                    } else {
                        LaunchType::Executable
                    },
                    executable_path: exe.clone(),
                    launch_args: Vec::new(),
                    icon_path: None,
                    package_id: provider.candidate_snap_names.first().cloned(),
                }));
            }
        }

        Ok(None)
    }

    fn is_process_running(&self, process: &ProcessIdentity) -> Result<bool, RuntimeError> {
        // Safe check via /proc filesystem without executing shell signals
        let proc_path = format!("/proc/{}", process.pid);
        Ok(Path::new(&proc_path).exists())
    }

    fn stop_process(
        &self,
        _process: &ProcessIdentity,
        _graceful_timeout_secs: u32,
        _allow_force: bool,
    ) -> Result<bool, RuntimeError> {
        // Process lifecycle stopping is strictly scoped for Milestone 4
        Err(RuntimeError::NotImplemented(
            "Safe process termination will be activated in Milestone 4".to_string(),
        ))
    }
}
