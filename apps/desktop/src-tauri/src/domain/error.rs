use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "code", content = "message")]
pub enum RuntimeError {
    ProviderNotFound(String),
    ApplicationNotInstalled(String),
    ApplicationAlreadyRunning(String),
    ApplicationNotRunning(String),
    LaunchFailed(String),
    ShutdownFailed(String),
    ShutdownTimeout(String),
    SwitchFailed(String),
    InvalidSettings(String),
    ProcessOwnershipUnknown(String),
    PlatformNotSupported(String),
    OperationInProgress(String),
    DiscoveryUnavailable(String),
    DiscoveryFailed(String),
    DiscoveryTimeout(String),
    InvalidMetadata(String),
    AmbiguousMatch(String),
    InvalidLaunchTarget(String),
    ProcessVerificationFailed(String),
    NotImplemented(String),
    Internal(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderNotFound(msg) => write!(f, "Provider not found: {msg}"),
            Self::ApplicationNotInstalled(msg) => write!(f, "Application not installed: {msg}"),
            Self::ApplicationAlreadyRunning(msg) => write!(f, "Application already running: {msg}"),
            Self::ApplicationNotRunning(msg) => write!(f, "Application not running: {msg}"),
            Self::LaunchFailed(msg) => write!(f, "Failed to launch application: {msg}"),
            Self::ShutdownFailed(msg) => write!(f, "Failed to stop application: {msg}"),
            Self::ShutdownTimeout(msg) => write!(f, "Shutdown timed out: {msg}"),
            Self::SwitchFailed(msg) => write!(f, "Failed to switch provider: {msg}"),
            Self::InvalidSettings(msg) => write!(f, "Invalid settings: {msg}"),
            Self::ProcessOwnershipUnknown(msg) => write!(f, "Process ownership unknown: {msg}"),
            Self::PlatformNotSupported(msg) => write!(f, "Platform not supported: {msg}"),
            Self::OperationInProgress(msg) => write!(f, "Operation in progress: {msg}"),
            Self::DiscoveryUnavailable(msg) => write!(f, "Discovery unavailable: {msg}"),
            Self::DiscoveryFailed(msg) => write!(f, "Discovery failed: {msg}"),
            Self::DiscoveryTimeout(msg) => write!(f, "Discovery timed out: {msg}"),
            Self::InvalidMetadata(msg) => write!(f, "Invalid metadata: {msg}"),
            Self::AmbiguousMatch(msg) => write!(f, "Ambiguous match: {msg}"),
            Self::InvalidLaunchTarget(msg) => write!(f, "Invalid launch target: {msg}"),
            Self::ProcessVerificationFailed(msg) => write!(f, "Process verification failed: {msg}"),
            Self::NotImplemented(msg) => write!(f, "Not implemented: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<RuntimeError> for String {
    fn from(err: RuntimeError) -> Self {
        serde_json::to_string(&err).unwrap_or_else(|_| err.to_string())
    }
}
