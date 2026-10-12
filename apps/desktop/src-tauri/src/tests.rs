use crate::{
    domain::{
        discovery::DiscoveryStatus, error::RuntimeError, lifecycle::ApplicationLifecycleState,
        provider::ProviderId, runtime::RuntimeSettings,
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

#[test]
fn test_snap_parser_valid_and_edge_cases() {
    use crate::runtime::snap::parse_snap_list_output;

    let sample_output = "\
Name             Version             Rev    Tracking       Publisher         Notes
bare             1.0                 5      latest/stable  canonical**       base
chatgpt-desktop  1.0.7               2      latest/stable  joshua-redmond    -
core22           20240408            1380   latest/stable  canonical**       base
gemini-desktop   0.6.0-1-g69bbec4ce7 11     latest/stable  ken-vandine*      -
gtk-common-themes 0.1-81-g442e3b1    1535   latest/stable/… canonical**       -
snapd            2.63                21759  latest/stable  canonical**       snapd
";

    let packages = parse_snap_list_output(sample_output).expect("Must parse valid snap output");
    assert_eq!(packages.len(), 6);

    let chatgpt = packages
        .iter()
        .find(|p| p.name == "chatgpt-desktop")
        .unwrap();
    assert_eq!(chatgpt.version, "1.0.7");
    assert_eq!(chatgpt.revision, "2");
    assert_eq!(chatgpt.tracking, "latest/stable");
    assert_eq!(chatgpt.publisher.as_deref(), Some("joshua-redmond"));
    assert_eq!(chatgpt.notes, None);

    let bare = packages.iter().find(|p| p.name == "bare").unwrap();
    assert_eq!(bare.notes.as_deref(), Some("base"));

    let gemini = packages
        .iter()
        .find(|p| p.name == "gemini-desktop")
        .unwrap();
    assert_eq!(gemini.version, "0.6.0-1-g69bbec4ce7");
    assert_eq!(gemini.publisher.as_deref(), Some("ken-vandine"));

    // Empty or header-only output
    assert_eq!(parse_snap_list_output("").unwrap().len(), 0);
    assert_eq!(parse_snap_list_output("   \n\n  ").unwrap().len(), 0);
    assert_eq!(
        parse_snap_list_output("Name Version Rev Tracking Publisher Notes\n")
            .unwrap()
            .len(),
        0
    );

    // Malformed line (too few columns) is skipped gracefully
    let malformed =
        "Name Version Rev Tracking Publisher Notes\ncorrupted_line\nvalid 1.0 1 stable pub -";
    let parsed_malformed = parse_snap_list_output(malformed).unwrap();
    assert_eq!(parsed_malformed.len(), 1);
    assert_eq!(parsed_malformed[0].name, "valid");
}

#[test]
fn test_match_provider_against_snaps_exact_and_ambiguous() {
    use crate::domain::discovery::{ApplicationSource, DiscoveryStatus, MatchConfidence};
    use crate::runtime::snap::{match_provider_against_snaps, parse_snap_list_output};

    let sample_output = "\
Name             Version             Rev    Tracking       Publisher         Notes
chatgpt-desktop  1.0.7               2      latest/stable  joshua-redmond    -
gemini-desktop   0.6.0               11     latest/stable  ken-vandine*      -
claude-community 2.1.0               4      latest/stable  someone           -
";
    let packages = parse_snap_list_output(sample_output).unwrap();
    let registry = ProviderRegistry::new();

    // 1. ChatGPT -> ExactCandidate
    let chatgpt_def = registry.get(ProviderId::ChatGPT).unwrap();
    let chatgpt_res = match_provider_against_snaps(chatgpt_def, &packages);
    assert_eq!(chatgpt_res.status, DiscoveryStatus::Confirmed);
    assert_eq!(chatgpt_res.match_confidence, MatchConfidence::Confirmed);
    assert_eq!(chatgpt_res.source, Some(ApplicationSource::Snap));
    assert_eq!(
        chatgpt_res.matched_package,
        Some("chatgpt-desktop".to_string())
    );
    assert_eq!(chatgpt_res.version, Some("1.0.7".to_string()));

    // 2. Gemini -> ExactCandidate
    let gemini_def = registry.get(ProviderId::Gemini).unwrap();
    let gemini_res = match_provider_against_snaps(gemini_def, &packages);
    assert_eq!(gemini_res.status, DiscoveryStatus::Confirmed);
    assert_eq!(gemini_res.match_confidence, MatchConfidence::Confirmed);
    assert_eq!(
        gemini_res.matched_package,
        Some("gemini-desktop".to_string())
    );

    // 3. Claude -> claude-community is not in known candidates, should be Ambiguous
    let claude_def = registry.get(ProviderId::Claude).unwrap();
    let claude_res = match_provider_against_snaps(claude_def, &packages);
    assert_eq!(claude_res.status, DiscoveryStatus::Ambiguous);
    assert_eq!(claude_res.match_confidence, MatchConfidence::Ambiguous);
    assert_eq!(
        claude_res.matched_package,
        Some("claude-community".to_string())
    );

    // 4. Perplexity -> Not installed
    let perplexity_def = registry.get(ProviderId::Perplexity).unwrap();
    let perplexity_res = match_provider_against_snaps(perplexity_def, &packages);
    assert_eq!(perplexity_res.status, DiscoveryStatus::NotInstalled);
    assert_eq!(perplexity_res.match_confidence, MatchConfidence::None);
    assert_eq!(perplexity_res.matched_package, None);
}

#[test]
fn test_generic_naming_guard_rejects_false_positives() {
    use crate::domain::discovery::{DiscoveryStatus, MatchConfidence};
    use crate::runtime::snap::{match_provider_against_snaps, parse_snap_list_output};

    // Packages with generic terms "ai", "chat", "assistant", "desktop"
    let generic_output = "\
Name             Version  Rev   Tracking       Publisher    Notes
ai-workbench     1.0      1     latest/stable  someorg      -
chat-client      2.0      2     latest/stable  someorg      -
desktop-tools    3.0      3     latest/stable  someorg      -
assistant-hub    4.0      4     latest/stable  someorg      -
gtk-common-themes 0.1     5     latest/stable  canonical**  -
";
    let packages = parse_snap_list_output(generic_output).unwrap();
    let registry = ProviderRegistry::new();

    // None of the providers should falsely match these generic names
    for provider in registry.list() {
        let res = match_provider_against_snaps(&provider, &packages);
        assert_eq!(
            res.status,
            DiscoveryStatus::NotInstalled,
            "Provider {} should not match generic package names",
            provider.name
        );
        assert_eq!(res.match_confidence, MatchConfidence::None);
    }
}

#[test]
fn test_discover_all_providers_lifecycle_updates_without_marking_running() {
    let runtime = Box::new(LinuxRuntime::new());
    let manager = AppLifecycleManager::new(runtime);

    let snapshot = manager
        .discover_all_providers()
        .expect("Discovery scan must succeed");
    assert_eq!(snapshot.results.len(), 6);

    // Verify all 6 providers in managed_apps
    for p in manager.list_providers() {
        let state = manager.get_provider_state(p.id).unwrap();
        // Crucial verification: Lifecycle state must NEVER be Running simply due to discovery
        assert_ne!(
            state.lifecycle_state,
            ApplicationLifecycleState::Running,
            "Provider {} must never be marked as Running due to discovery",
            p.id.as_str()
        );
        assert!(
            state.process.is_none(),
            "ProcessIdentity must remain None during discovery"
        );
    }

    // On this host, chatgpt-desktop and gemini-desktop are real installed snaps!
    // Check if chatgpt or gemini are confirmed when snap is available
    if snapshot.is_snap_available {
        let chatgpt_res = snapshot
            .results
            .iter()
            .find(|r| r.provider_id == ProviderId::ChatGPT)
            .unwrap();
        assert_eq!(chatgpt_res.status, DiscoveryStatus::Confirmed);
        assert_eq!(
            chatgpt_res.matched_package,
            Some("chatgpt-desktop".to_string())
        );

        let chatgpt_state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
        assert_eq!(
            chatgpt_state.lifecycle_state,
            ApplicationLifecycleState::Installed
        );

        let gemini_res = snapshot
            .results
            .iter()
            .find(|r| r.provider_id == ProviderId::Gemini)
            .unwrap();
        assert_eq!(gemini_res.status, DiscoveryStatus::Confirmed);
        assert_eq!(
            gemini_res.matched_package,
            Some("gemini-desktop".to_string())
        );

        let gemini_state = manager.get_provider_state(ProviderId::Gemini).unwrap();
        assert_eq!(
            gemini_state.lifecycle_state,
            ApplicationLifecycleState::Installed
        );
    }
}
