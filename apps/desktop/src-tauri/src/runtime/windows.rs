use super::traits::RuntimePlatform;
use crate::domain::{
    discovery::{DiscoverySnapshot, InstalledApplication},
    error::RuntimeError,
    launch::LaunchResult,
    lifecycle::ProcessIdentity,
    provider::ProviderDefinition,
    ResourceSnapshot,
};

#[derive(Debug, Default)]
pub struct WindowsRuntime;

impl WindowsRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl RuntimePlatform for WindowsRuntime {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn discover_all_applications(
        &self,
        _providers: &[ProviderDefinition],
    ) -> Result<DiscoverySnapshot, RuntimeError> {
        #[cfg(target_os = "windows")]
        {
            Ok(DiscoverySnapshot {
                is_snap_available: false,
                scanned_at_ms: 0,
                total_packages_scanned: 0,
                results: Vec::new(),
                diagnostics: Some("Snap discovery is not applicable on Windows".to_string()),
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(RuntimeError::PlatformNotSupported(
                "Windows runtime is not supported on this host operating system".to_string(),
            ))
        }
    }

    fn detect_application(
        &self,
        _provider: &ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        #[cfg(target_os = "windows")]
        {
            // Windows implementation will be introduced and tested on Windows target in Milestone 8
            Ok(None)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(RuntimeError::PlatformNotSupported(
                "Windows runtime is not supported on this host operating system".to_string(),
            ))
        }
    }

    fn launch_application(
        &self,
        _app: &InstalledApplication,
    ) -> Result<LaunchResult, RuntimeError> {
        #[cfg(target_os = "windows")]
        {
            Err(RuntimeError::PlatformNotSupported(
                "Windows native application launching requires verified Windows discovery (Milestone 8)".to_string(),
            ))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(RuntimeError::PlatformNotSupported(
                "Windows runtime cannot launch applications on a Linux host".to_string(),
            ))
        }
    }

    fn is_process_running(&self, _process: &ProcessIdentity) -> Result<bool, RuntimeError> {
        #[cfg(target_os = "windows")]
        {
            Ok(false)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(RuntimeError::PlatformNotSupported(
                "Windows runtime is not supported on this host operating system".to_string(),
            ))
        }
    }

    fn stop_process(
        &self,
        _process: &ProcessIdentity,
        _graceful_timeout_secs: u32,
        _allow_force: bool,
    ) -> Result<bool, RuntimeError> {
        Err(RuntimeError::PlatformNotSupported(
            "Windows process management is not available on this host".to_string(),
        ))
    }

    fn validate_process(
        &self,
        _process: &ProcessIdentity,
        _provider: &ProviderDefinition,
    ) -> Result<bool, RuntimeError> {
        Err(RuntimeError::PlatformNotSupported(
            "Windows process identity validation is not supported on this host".to_string(),
        ))
    }

    fn get_system_resources(&self) -> Result<ResourceSnapshot, RuntimeError> {
        Ok(ResourceSnapshot {
            is_available: false,
            diagnostics: Some(
                "System resource monitoring is not supported on Windows in this build".to_string(),
            ),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            ..Default::default()
        })
    }
}
