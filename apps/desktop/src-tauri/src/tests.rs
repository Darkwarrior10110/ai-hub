use crate::{
    domain::{
        error::RuntimeError, lifecycle::ApplicationLifecycleState, provider::ProviderId,
        runtime::RuntimeSettings,
    },
    manager::AppLifecycleManager,
    registry::ProviderRegistry,
    runtime::LinuxRuntime,
};
use std::str::FromStr;

#[test]
fn test_provider_registry_contains_all_six_providers() {
    let registry = ProviderRegistry::new();
    assert_eq!(registry.len(), 6);

    let providers = registry.list();
    assert_eq!(providers.len(), 6);

    let ids: Vec<ProviderId> = providers.into_iter().map(|p| p.id).collect();
    assert_eq!(
        ids,
        vec![
            ProviderId::ChatGPT,
            ProviderId::Gemini,
            ProviderId::Claude,
            ProviderId::Grok,
            ProviderId::Perplexity,
            ProviderId::Kimi,
        ]
    );
}

#[test]
fn test_provider_id_parsing_and_validation() {
    assert_eq!(
        ProviderId::from_str("chatgpt").unwrap(),
        ProviderId::ChatGPT
    );
    assert_eq!(ProviderId::from_str("GEMINI").unwrap(), ProviderId::Gemini);
    assert_eq!(ProviderId::from_str("claude").unwrap(), ProviderId::Claude);
    assert_eq!(ProviderId::from_str("grok").unwrap(), ProviderId::Grok);
    assert_eq!(
        ProviderId::from_str("perplexity").unwrap(),
        ProviderId::Perplexity
    );
    assert_eq!(ProviderId::from_str("kimi").unwrap(), ProviderId::Kimi);

    let err = ProviderId::from_str("unknown_ai");
    assert!(matches!(err, Err(RuntimeError::ProviderNotFound(_))));
}

#[test]
fn test_runtime_settings_validation() {
    let default_settings = RuntimeSettings::default();
    assert!(default_settings.validate().is_ok());
    assert!(default_settings.resource_saver_mode);
    assert!(
        !default_settings.allow_force_termination,
        "Force termination must be disabled by default"
    );

    let invalid_timeout = RuntimeSettings {
        graceful_shutdown_timeout_secs: 0,
        ..default_settings.clone()
    };
    assert!(matches!(
        invalid_timeout.validate(),
        Err(RuntimeError::InvalidSettings(_))
    ));

    let invalid_timeout_high = RuntimeSettings {
        graceful_shutdown_timeout_secs: 100,
        ..default_settings.clone()
    };
    assert!(matches!(
        invalid_timeout_high.validate(),
        Err(RuntimeError::InvalidSettings(_))
    ));

    let invalid_polling = RuntimeSettings {
        metrics_polling_interval_secs: 0,
        ..default_settings
    };
    assert!(matches!(
        invalid_polling.validate(),
        Err(RuntimeError::InvalidSettings(_))
    ));
}

#[test]
fn test_manager_lifecycle_states_and_active_switching() {
    let runtime = Box::new(LinuxRuntime::new());
    let manager = AppLifecycleManager::new(runtime);

    // Initial providers list
    let list = manager.list_providers();
    assert_eq!(list.len(), 6);

    // Initial state of chatgpt
    let state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(state.provider_id, ProviderId::ChatGPT);
    assert!(!state.is_active);
    assert!(matches!(
        state.lifecycle_state,
        ApplicationLifecycleState::Installed
            | ApplicationLifecycleState::NotInstalled
            | ApplicationLifecycleState::Unknown
    ));

    // Active provider setting
    assert_eq!(manager.get_active_provider(), None);
    manager.set_active_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(manager.get_active_provider(), Some(ProviderId::ChatGPT));

    let updated_state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert!(updated_state.is_active);

    // Switching active provider to Gemini
    manager.set_active_provider(ProviderId::Gemini).unwrap();
    assert_eq!(manager.get_active_provider(), Some(ProviderId::Gemini));

    let chatgpt_state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    let gemini_state = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert!(
        !chatgpt_state.is_active,
        "ChatGPT should no longer be active"
    );
    assert!(gemini_state.is_active, "Gemini should now be active");
}

#[test]
fn test_manager_launch_and_stop_structured_errors_in_milestone_1() {
    let runtime = Box::new(LinuxRuntime::new());
    let manager = AppLifecycleManager::new(runtime);

    // Launching returns structured NotImplemented without fabricating fake runs
    let launch_res = manager.launch_provider(ProviderId::ChatGPT);
    assert!(matches!(launch_res, Err(RuntimeError::NotImplemented(_))));

    let stop_res = manager.stop_provider(ProviderId::ChatGPT);
    assert!(matches!(stop_res, Err(RuntimeError::NotImplemented(_))));

    let switch_res = manager.switch_provider(ProviderId::Gemini);
    assert!(matches!(switch_res, Err(RuntimeError::NotImplemented(_))));
}
