use super::traits::RuntimePlatform;
use crate::domain::{
    discovery::InstalledApplication,
    error::RuntimeError,
    lifecycle::ProcessIdentity,
    provider::{LaunchType, ProviderDefinition},
};
use std::path::Path;

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

    fn detect_application(
        &self,
        provider: &ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        // Safe inspection of candidate executables and snap desktop locations
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

        // Check common snap desktop files
        for desktop_id in &provider.candidate_desktop_ids {
            let snap_desktop = format!("/var/lib/snapd/desktop/applications/{desktop_id}");
            if Path::new(&snap_desktop).exists() {
                let snap_exe = provider
                    .candidate_executables
                    .first()
                    .cloned()
                    .unwrap_or_else(|| format!("/snap/bin/{}", provider.id.as_str()));

                return Ok(Some(InstalledApplication {
                    provider_id: provider.id,
                    display_name: provider.name.clone(),
                    version: None,
                    launch_type: LaunchType::Snap,
                    executable_path: snap_exe,
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
