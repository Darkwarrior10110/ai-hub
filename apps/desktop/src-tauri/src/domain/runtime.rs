use super::error::RuntimeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSettings {
    pub resource_saver_mode: bool,
    pub confirm_before_switching: bool,
    pub graceful_shutdown_timeout_secs: u32,
    pub allow_force_termination: bool,
    pub auto_start_on_boot: bool,
    pub metrics_polling_interval_secs: u32,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            resource_saver_mode: true,
            confirm_before_switching: true,
            graceful_shutdown_timeout_secs: 5,
            allow_force_termination: false, // Explicitly disabled by default per safety requirement
            auto_start_on_boot: false,
            metrics_polling_interval_secs: 3,
        }
    }
}

impl RuntimeSettings {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        if self.graceful_shutdown_timeout_secs < 1 || self.graceful_shutdown_timeout_secs > 60 {
            return Err(RuntimeError::InvalidSettings(
                "Graceful shutdown timeout must be between 1 and 60 seconds".to_string(),
            ));
        }

        if self.metrics_polling_interval_secs < 1 || self.metrics_polling_interval_secs > 300 {
            return Err(RuntimeError::InvalidSettings(
                "Metrics polling interval must be between 1 and 300 seconds".to_string(),
            ));
        }

        Ok(())
    }
}
