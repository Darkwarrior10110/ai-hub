use crate::domain::{
    discovery::{DiscoverySnapshot, InstalledApplication},
    error::RuntimeError,
    launch::LaunchResult,
    lifecycle::ProcessIdentity,
    provider::ProviderDefinition,
    resource::ResourceSnapshot,
};

pub trait RuntimePlatform: Send + Sync {
    fn name(&self) -> &'static str;

    fn detect_application(
        &self,
        provider: &ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError>;

    fn discover_all_applications(
        &self,
        providers: &[ProviderDefinition],
    ) -> Result<DiscoverySnapshot, RuntimeError>;

    fn launch_application(&self, app: &InstalledApplication) -> Result<LaunchResult, RuntimeError>;

    fn is_process_running(&self, process: &ProcessIdentity) -> Result<bool, RuntimeError>;

    fn validate_process(
        &self,
        process: &ProcessIdentity,
        provider: &ProviderDefinition,
    ) -> Result<bool, RuntimeError>;

    fn stop_process(
        &self,
        process: &ProcessIdentity,
        graceful_timeout_secs: u32,
        allow_force: bool,
    ) -> Result<bool, RuntimeError>;

    fn get_system_resources(&self) -> Result<ResourceSnapshot, RuntimeError>;
}
