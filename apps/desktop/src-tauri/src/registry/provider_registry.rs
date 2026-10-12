use crate::domain::{
    error::RuntimeError,
    provider::{ProviderDefinition, ProviderId, SupportedPlatform},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProviderRegistry {
    providers: HashMap<ProviderId, ProviderDefinition>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
        };
        registry.register_default_catalog();
        registry
    }

    fn register_default_catalog(&mut self) {
        // 1. ChatGPT
        self.register(ProviderDefinition {
            id: ProviderId::ChatGPT,
            name: "ChatGPT".to_string(),
            description: "OpenAI ChatGPT desktop application".to_string(),
            vendor: "OpenAI".to_string(),
            accent_color: "#10a37f".to_string(),
            supported_platforms: vec![
                SupportedPlatform::Linux,
                SupportedPlatform::Windows,
                SupportedPlatform::MacOS,
            ],
            candidate_snap_names: vec!["chatgpt-desktop".to_string(), "chatgpt".to_string()],
            candidate_desktop_ids: vec![
                "chatgpt-desktop_chatgpt-desktop.desktop".to_string(),
                "chatgpt.desktop".to_string(),
            ],
            candidate_executables: vec![
                "/snap/bin/chatgpt-desktop".to_string(),
                "chatgpt-desktop".to_string(),
                "chatgpt".to_string(),
            ],
        });

        // 2. Gemini
        self.register(ProviderDefinition {
            id: ProviderId::Gemini,
            name: "Gemini".to_string(),
            description: "Google Gemini desktop application".to_string(),
            vendor: "Google".to_string(),
            accent_color: "#4285f4".to_string(),
            supported_platforms: vec![
                SupportedPlatform::Linux,
                SupportedPlatform::Windows,
                SupportedPlatform::MacOS,
            ],
            candidate_snap_names: vec!["gemini-desktop".to_string(), "gemini".to_string()],
            candidate_desktop_ids: vec![
                "gemini-desktop_gemini-desktop.desktop".to_string(),
                "gemini.desktop".to_string(),
            ],
            candidate_executables: vec![
                "/snap/bin/gemini-desktop".to_string(),
                "gemini-desktop".to_string(),
                "gemini".to_string(),
            ],
        });

        // 3. Claude
        self.register(ProviderDefinition {
            id: ProviderId::Claude,
            name: "Claude".to_string(),
            description: "Anthropic Claude desktop application".to_string(),
            vendor: "Anthropic".to_string(),
            accent_color: "#d97706".to_string(),
            supported_platforms: vec![
                SupportedPlatform::Linux,
                SupportedPlatform::Windows,
                SupportedPlatform::MacOS,
            ],
            candidate_snap_names: vec!["claude-desktop".to_string(), "claude".to_string()],
            candidate_desktop_ids: vec![
                "claude-desktop.desktop".to_string(),
                "claude.desktop".to_string(),
            ],
            candidate_executables: vec!["claude-desktop".to_string(), "claude".to_string()],
        });

        // 4. Grok
        self.register(ProviderDefinition {
            id: ProviderId::Grok,
            name: "Grok".to_string(),
            description: "xAI Grok desktop application".to_string(),
            vendor: "xAI".to_string(),
            accent_color: "#1d9bf0".to_string(),
            supported_platforms: vec![
                SupportedPlatform::Linux,
                SupportedPlatform::Windows,
                SupportedPlatform::MacOS,
            ],
            candidate_snap_names: vec!["grok-desktop".to_string(), "grok".to_string()],
            candidate_desktop_ids: vec![
                "grok-desktop.desktop".to_string(),
                "grok.desktop".to_string(),
            ],
            candidate_executables: vec!["grok-desktop".to_string(), "grok".to_string()],
        });

        // 5. Perplexity
        self.register(ProviderDefinition {
            id: ProviderId::Perplexity,
            name: "Perplexity".to_string(),
            description: "Perplexity AI desktop application".to_string(),
            vendor: "Perplexity AI".to_string(),
            accent_color: "#20b2aa".to_string(),
            supported_platforms: vec![
                SupportedPlatform::Linux,
                SupportedPlatform::Windows,
                SupportedPlatform::MacOS,
            ],
            candidate_snap_names: vec!["perplexity-desktop".to_string(), "perplexity".to_string()],
            candidate_desktop_ids: vec![
                "perplexity-desktop.desktop".to_string(),
                "perplexity.desktop".to_string(),
            ],
            candidate_executables: vec!["perplexity-desktop".to_string(), "perplexity".to_string()],
        });

        // 6. Kimi
        self.register(ProviderDefinition {
            id: ProviderId::Kimi,
            name: "Kimi".to_string(),
            description: "Moonshot AI Kimi desktop application".to_string(),
            vendor: "Moonshot AI".to_string(),
            accent_color: "#7c3aed".to_string(),
            supported_platforms: vec![
                SupportedPlatform::Linux,
                SupportedPlatform::Windows,
                SupportedPlatform::MacOS,
            ],
            candidate_snap_names: vec!["kimi-desktop".to_string(), "kimi".to_string()],
            candidate_desktop_ids: vec![
                "kimi-desktop.desktop".to_string(),
                "kimi.desktop".to_string(),
            ],
            candidate_executables: vec!["kimi-desktop".to_string(), "kimi".to_string()],
        });
    }

    pub fn register(&mut self, definition: ProviderDefinition) {
        self.providers.insert(definition.id, definition);
    }

    pub fn get(&self, id: ProviderId) -> Result<&ProviderDefinition, RuntimeError> {
        self.providers.get(&id).ok_or_else(|| {
            RuntimeError::ProviderNotFound(format!("Provider {} not found in catalog", id.as_str()))
        })
    }

    pub fn list(&self) -> Vec<ProviderDefinition> {
        // Return in canonical order
        ProviderId::ALL
            .iter()
            .filter_map(|id| self.providers.get(id).cloned())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.providers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}
