use crate::domain::{
    discovery::InstalledApplication, error::RuntimeError, lifecycle::ProcessIdentity,
    provider::ProviderDefinition,
};

pub trait RuntimePlatform: Send + Sync {
    fn name(&self) -> &'static str;

    fn detect_application(
        &self,
        provider: &ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError>;

    fn is_process_running(&self, process: &ProcessIdentity) -> Result<bool, RuntimeError>;

    fn stop_process(
        &self,
        process: &ProcessIdentity,
        graceful_timeout_secs: u32,
        allow_force: bool,
    ) -> Result<bool, RuntimeError>;
}
