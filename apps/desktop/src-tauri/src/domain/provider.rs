use super::error::RuntimeError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    ChatGPT,
    Gemini,
    Claude,
    Grok,
    Perplexity,
    Kimi,
}

impl ProviderId {
    pub const ALL: [ProviderId; 6] = [
        ProviderId::ChatGPT,
        ProviderId::Gemini,
        ProviderId::Claude,
        ProviderId::Grok,
        ProviderId::Perplexity,
        ProviderId::Kimi,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChatGPT => "chatgpt",
            Self::Gemini => "gemini",
            Self::Claude => "claude",
            Self::Grok => "grok",
            Self::Perplexity => "perplexity",
            Self::Kimi => "kimi",
        }
    }
}

impl FromStr for ProviderId {
    type Err = RuntimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "chatgpt" => Ok(Self::ChatGPT),
            "gemini" => Ok(Self::Gemini),
            "claude" => Ok(Self::Claude),
            "grok" => Ok(Self::Grok),
            "perplexity" => Ok(Self::Perplexity),
            "kimi" => Ok(Self::Kimi),
            other => Err(RuntimeError::ProviderNotFound(format!(
                "Unknown provider identifier: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportedPlatform {
    Linux,
    Windows,
    MacOS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchType {
    Snap,
    DesktopEntry,
    Executable,
    AppImage,
    WindowsApp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderDefinition {
    pub id: ProviderId,
    pub name: String,
    pub description: String,
    pub vendor: String,
    pub accent_color: String,
    pub supported_platforms: Vec<SupportedPlatform>,
    pub candidate_snap_names: Vec<String>,
    pub candidate_desktop_ids: Vec<String>,
    pub candidate_executables: Vec<String>,
}
