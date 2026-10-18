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
use std::sync::{Arc, RwLock};

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
fn test_manager_stop_and_switch_structured_errors() {
    let runtime = Box::new(LinuxRuntime::new());
    let manager = AppLifecycleManager::new(runtime);

    // In Milestone 4/5, stop_provider is implemented: when not running, returns ApplicationNotRunning
    let stop_res = manager.stop_provider(ProviderId::ChatGPT);
    assert!(matches!(
        stop_res,
        Err(RuntimeError::ApplicationNotRunning(_))
    ));

    // In Milestone 5, switch_provider to uninstalled app returns ApplicationNotInstalled
    let switch_res = manager.switch_provider(ProviderId::Kimi);
    assert!(matches!(
        switch_res,
        Err(RuntimeError::ApplicationNotInstalled(_))
    ));
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
    // Check if chatgpt or gemini are confirmed when snap query succeeded without timeout
    if snapshot.is_snap_available && snapshot.diagnostics.is_none() {
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

// ============================================================================
// Milestone 3: Safe Application Launching Tests
// ============================================================================

use crate::domain::{
    discovery::InstalledApplication,
    launch::{LaunchRequest, LaunchResult, LaunchStatus},
    lifecycle::{ProcessIdentity, ProcessOwnership},
    provider::LaunchType,
};
use crate::runtime::{traits::RuntimePlatform, ProcessLauncher, WindowsRuntime};
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Debug)]
struct MockProcessLauncher {
    spawn_result: Mutex<Result<u32, RuntimeError>>,
    running_pids: Mutex<HashSet<u32>>,
}

impl MockProcessLauncher {
    fn new(pid: u32, is_running: bool) -> Self {
        let mut pids = HashSet::new();
        if is_running {
            pids.insert(pid);
        }
        Self {
            spawn_result: Mutex::new(Ok(pid)),
            running_pids: Mutex::new(pids),
        }
    }
}

impl ProcessLauncher for MockProcessLauncher {
    fn spawn(&self, _executable: &str, _args: &[String]) -> Result<u32, RuntimeError> {
        self.spawn_result.lock().unwrap().clone()
    }

    fn is_running(&self, pid: u32) -> bool {
        self.running_pids.lock().unwrap().contains(&pid)
    }

    fn inspect_process(
        &self,
        pid: u32,
    ) -> Result<crate::runtime::linux::ProcessInspection, RuntimeError> {
        let is_run = self.is_running(pid);
        Ok(crate::runtime::linux::ProcessInspection {
            pid,
            exists: is_run,
            uid: if is_run { Some(1000) } else { None },
            start_time_ticks: if is_run { Some(12345) } else { None },
            comm: if is_run {
                Some("chatgpt-desktop".to_string())
            } else {
                None
            },
            cmdline: if is_run {
                Some("/snap/bin/chatgpt-desktop".to_string())
            } else {
                None
            },
            exe_path: if is_run {
                Some("/snap/bin/chatgpt-desktop".to_string())
            } else {
                None
            },
            state: if is_run { Some('S') } else { None },
            children: Vec::new(),
        })
    }

    fn send_signal(&self, pid: u32, _signal: i32) -> Result<(), RuntimeError> {
        self.running_pids.lock().unwrap().remove(&pid);
        Ok(())
    }

    fn get_current_uid(&self) -> u32 {
        1000
    }

    fn read_meminfo(&self) -> Result<String, RuntimeError> {
        Ok("MemTotal:        4000000 kB\nMemFree:          500000 kB\nMemAvailable:    1200000 kB\nBuffers:          100000 kB\nCached:           600000 kB\nSwapTotal:       2000000 kB\nSwapFree:        1500000 kB\n".to_string())
    }
}

#[derive(Debug)]
struct MockRuntimePlatform {
    detect_res: Mutex<Result<Option<InstalledApplication>, RuntimeError>>,
    launch_res: Mutex<Result<LaunchResult, RuntimeError>>,
    running_pids: Mutex<HashSet<u32>>,
}

impl MockRuntimePlatform {
    fn new(
        detect: Result<Option<InstalledApplication>, RuntimeError>,
        launch: Result<LaunchResult, RuntimeError>,
        running_pids: HashSet<u32>,
    ) -> Self {
        Self {
            detect_res: Mutex::new(detect),
            launch_res: Mutex::new(launch),
            running_pids: Mutex::new(running_pids),
        }
    }
}

impl RuntimePlatform for MockRuntimePlatform {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn detect_application(
        &self,
        _provider: &crate::domain::provider::ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        self.detect_res.lock().unwrap().clone()
    }

    fn discover_all_applications(
        &self,
        _providers: &[crate::domain::provider::ProviderDefinition],
    ) -> Result<crate::domain::discovery::DiscoverySnapshot, RuntimeError> {
        Ok(crate::domain::discovery::DiscoverySnapshot {
            is_snap_available: true,
            scanned_at_ms: 1000,
            total_packages_scanned: 1,
            results: Vec::new(),
            diagnostics: None,
        })
    }

    fn launch_application(
        &self,
        _app: &InstalledApplication,
    ) -> Result<LaunchResult, RuntimeError> {
        self.launch_res.lock().unwrap().clone()
    }

    fn is_process_running(&self, process: &ProcessIdentity) -> Result<bool, RuntimeError> {
        Ok(self.running_pids.lock().unwrap().contains(&process.pid))
    }

    fn stop_process(
        &self,
        process: &ProcessIdentity,
        _graceful_timeout_secs: u32,
        _allow_force: bool,
    ) -> Result<bool, RuntimeError> {
        let mut pids = self.running_pids.lock().unwrap();
        if pids.contains(&process.pid) {
            pids.remove(&process.pid);
            Ok(true)
        } else {
            Ok(true)
        }
    }

    fn validate_process(
        &self,
        process: &ProcessIdentity,
        _provider: &crate::domain::provider::ProviderDefinition,
    ) -> Result<bool, RuntimeError> {
        Ok(self.running_pids.lock().unwrap().contains(&process.pid))
    }

    fn get_system_resources(&self) -> Result<crate::domain::ResourceSnapshot, RuntimeError> {
        Ok(crate::domain::ResourceSnapshot {
            is_available: true,
            system_memory: Some(crate::domain::resource::SystemMemory {
                total_bytes: 4 * 1024 * 1024 * 1024,
                available_bytes: 2 * 1024 * 1024 * 1024,
                used_bytes: 2 * 1024 * 1024 * 1024,
                used_percentage: 50,
                swap_total_bytes: 2 * 1024 * 1024 * 1024,
                swap_free_bytes: 1500 * 1024 * 1024,
                swap_used_bytes: 500 * 1024 * 1024,
            }),
            memory_pressure_warning: false,
            memory_pressure_message: None,
            tracked_processes_count: 0,
            process_metrics: Vec::new(),
            diagnostics: Some("Mock system resources".to_string()),
            timestamp_ms: 1000,
            ..Default::default()
        })
    }
}

#[test]
fn test_launch_result_and_request_serialization() {
    let req = LaunchRequest {
        provider_id: ProviderId::ChatGPT,
        launch_type: Some(LaunchType::Snap),
        custom_args: Some(vec!["--verbose".to_string()]),
    };
    let json = serde_json::to_string(&req).expect("Serialize LaunchRequest");
    assert!(json.contains("\"provider_id\":\"chatgpt\""));
    assert!(json.contains("\"launch_type\":\"snap\""));

    let res = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 12345,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1700000000000),
            start_time_ticks: Some(123456),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned successfully".to_string()),
        launched_at_ms: 1700000000000,
    };
    let res_json = serde_json::to_string(&res).expect("Serialize LaunchResult");
    assert!(res_json.contains("\"status\":\"running\""));
    assert!(res_json.contains("\"pid\":12345"));
    assert!(res_json.contains("\"ownership\":\"started_by_ai_hub\""));
}

#[test]
fn test_linux_runtime_launch_with_mock_launcher() {
    let mock_launcher = Box::new(MockProcessLauncher::new(9999, true));
    let runtime = LinuxRuntime::with_launcher(mock_launcher);

    let valid_exe = if std::path::Path::new("/snap/bin/chatgpt-desktop").exists() {
        "/snap/bin/chatgpt-desktop".to_string()
    } else {
        "/bin/sh".to_string()
    };

    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: if valid_exe.starts_with("/snap/") {
            LaunchType::Snap
        } else {
            LaunchType::Executable
        },
        executable_path: valid_exe,
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let result = runtime
        .launch_application(&app)
        .expect("Launch must succeed");
    assert_eq!(result.provider_id, ProviderId::ChatGPT);
    assert_eq!(result.status, LaunchStatus::Running);
    assert!(result.process.is_some());
    let proc = result.process.unwrap();
    assert_eq!(proc.pid, 9999);
    assert_eq!(proc.ownership, ProcessOwnership::StartedByAIHub);
}

#[test]
fn test_linux_runtime_security_validations() {
    let mock_launcher = Box::new(MockProcessLauncher::new(1234, true));
    let runtime = LinuxRuntime::with_launcher(mock_launcher);

    // 1. Relative path rejected
    let rel_app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: None,
        launch_type: LaunchType::Executable,
        executable_path: "chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: None,
    };
    let rel_err = runtime.launch_application(&rel_app);
    assert!(matches!(rel_err, Err(RuntimeError::InvalidLaunchTarget(_))));

    // 2. Non-existent path rejected
    let non_existent_app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: None,
        launch_type: LaunchType::Executable,
        executable_path: "/non/existent/path/to/binary".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: None,
    };
    let non_existent_err = runtime.launch_application(&non_existent_app);
    assert!(matches!(
        non_existent_err,
        Err(RuntimeError::LaunchFailed(_))
    ));

    // 3. Command injection rejected
    let injection_app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: None,
        launch_type: LaunchType::Executable,
        executable_path: "/bin/echo; rm -rf /".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: None,
    };
    let injection_err = runtime.launch_application(&injection_app);
    assert!(matches!(
        injection_err,
        Err(RuntimeError::InvalidLaunchTarget(_))
    ));
}

#[test]
fn test_manager_launch_success_and_state_updates() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 5555,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(10000),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(5555);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);

    // Initial state
    let state_before = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        state_before.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
    assert_eq!(state_before.process, None);

    // Launch
    let res = manager
        .launch_provider(ProviderId::ChatGPT)
        .expect("Launch succeeds");
    assert_eq!(res.status, LaunchStatus::Running);
    assert_eq!(res.process.as_ref().unwrap().pid, 5555);

    // State after launch
    let state_after = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        state_after.lifecycle_state,
        ApplicationLifecycleState::Running
    );
    assert_eq!(state_after.process.as_ref().unwrap().pid, 5555);
    assert_eq!(
        state_after.process.as_ref().unwrap().ownership,
        ProcessOwnership::StartedByAIHub
    );
    assert!(state_after.is_active);
    assert_eq!(manager.get_active_provider(), Some(ProviderId::ChatGPT));
}

#[test]
fn test_manager_launch_already_running_does_not_spawn_duplicate() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 7777,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(2000),
            start_time_ticks: Some(20000),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 2000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(7777);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);

    // First launch succeeds
    let first = manager.launch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(first.status, LaunchStatus::Running);

    // Second launch detects already running instance and returns AlreadyRunning without re-spawning
    let second = manager.launch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(second.status, LaunchStatus::AlreadyRunning);
    assert_eq!(second.process.as_ref().unwrap().pid, 7777);
}

#[test]
fn test_manager_launch_stale_pid_cleanup_and_recovery() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 8888,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(3000),
            start_time_ticks: Some(30000),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 3000,
    };

    // Note: PID 8888 is NOT in running_pids (simulating crashed or externally killed process)
    let running_pids = HashSet::new();

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);

    // First launch sets running state
    let _ = manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Now, is_process_running is false for PID 8888. Second launch should detect stale PID,
    // clean it up, and re-launch
    let second = manager.launch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(second.status, LaunchStatus::Running);
}

#[test]
fn test_manager_launch_ambiguous_match_rejected() {
    let runtime = Box::new(MockRuntimePlatform::new(
        Err(RuntimeError::AmbiguousMatch(
            "Multiple packages matched claude".to_string(),
        )),
        Err(RuntimeError::Internal("Should not be called".to_string())),
        HashSet::new(),
    ));

    let manager = AppLifecycleManager::new(runtime);

    let res = manager.launch_provider(ProviderId::Claude);
    assert!(matches!(res, Err(RuntimeError::AmbiguousMatch(_))));

    let state = manager.get_provider_state(ProviderId::Claude).unwrap();
    assert_eq!(state.lifecycle_state, ApplicationLifecycleState::Error);
    assert!(state.error_message.unwrap().contains("Ambiguous"));
}

#[test]
fn test_manager_launch_not_installed_rejected() {
    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(None),
        Err(RuntimeError::Internal("Should not be called".to_string())),
        HashSet::new(),
    ));

    let manager = AppLifecycleManager::new(runtime);

    let res = manager.launch_provider(ProviderId::Perplexity);
    assert!(matches!(res, Err(RuntimeError::ApplicationNotInstalled(_))));

    let state = manager.get_provider_state(ProviderId::Perplexity).unwrap();
    assert_eq!(
        state.lifecycle_state,
        ApplicationLifecycleState::NotInstalled
    );
}

#[test]
fn test_windows_runtime_launch_unsupported_on_linux() {
    let runtime = WindowsRuntime::new();
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: None,
        launch_type: LaunchType::WindowsApp,
        executable_path: "C:\\Program Files\\ChatGPT\\ChatGPT.exe".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: None,
    };

    let res = runtime.launch_application(&app);
    assert!(matches!(res, Err(RuntimeError::PlatformNotSupported(_))));
}

#[test]
fn test_parse_stat_starttime_handles_spaces_and_parentheses() {
    use crate::runtime::linux::parse_stat_starttime;

    // Normal comm
    let normal_stat = "400792 (cat) R 389815 400792 400792 34823 400792 4194304 276 0 0 0 0 0 0 0 20 0 1 0 36420620 8302592";
    assert_eq!(parse_stat_starttime(normal_stat), Some(36420620));

    // Comm containing spaces and nested parentheses
    let complex_stat = "12345 (chatgpt (worker) helper) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 987654321 8302592";
    assert_eq!(parse_stat_starttime(complex_stat), Some(987654321));

    // Truncated stat line
    let truncated_stat = "12345 (cat) S 1 2 3";
    assert_eq!(parse_stat_starttime(truncated_stat), None);
}

#[test]
fn test_process_identity_matching_and_rejection() {
    use crate::runtime::linux::{is_process_matching_provider, ProcessInspection};

    let registry = ProviderRegistry::new();
    let chatgpt_def = registry.get(ProviderId::ChatGPT).unwrap();
    let gemini_def = registry.get(ProviderId::Gemini).unwrap();

    // 1. Exact match on comm
    let matching_comm = ProcessInspection {
        pid: 100,
        exists: true,
        uid: Some(1000),
        start_time_ticks: Some(500),
        comm: Some("chatgpt-desktop".to_string()),
        cmdline: Some("/snap/bin/chatgpt-desktop".to_string()),
        exe_path: Some("/snap/chatgpt-desktop/12/chatgpt".to_string()),
        state: Some('S'),
        children: Vec::new(),
    };
    assert!(is_process_matching_provider(
        &matching_comm,
        chatgpt_def,
        None
    ));
    assert!(!is_process_matching_provider(
        &matching_comm,
        gemini_def,
        None
    ));

    // 2. Exact match on snap directory in exe_path
    let matching_exe = ProcessInspection {
        pid: 101,
        exists: true,
        uid: Some(1000),
        start_time_ticks: Some(500),
        comm: Some("gemini".to_string()),
        cmdline: None,
        exe_path: Some("/snap/gemini-desktop/current/gemini".to_string()),
        state: Some('S'),
        children: Vec::new(),
    };
    assert!(is_process_matching_provider(
        &matching_exe,
        gemini_def,
        None
    ));
    assert!(!is_process_matching_provider(
        &matching_exe,
        chatgpt_def,
        None
    ));

    // 3. Rejection of unrelated system process (no false positive)
    let unrelated_proc = ProcessInspection {
        pid: 102,
        exists: true,
        uid: Some(1000),
        start_time_ticks: Some(500),
        comm: Some("gnome-terminal".to_string()),
        cmdline: Some("/usr/bin/gnome-terminal-server".to_string()),
        exe_path: Some("/usr/libexec/gnome-terminal-server".to_string()),
        state: Some('S'),
        children: Vec::new(),
    };
    assert!(!is_process_matching_provider(
        &unrelated_proc,
        chatgpt_def,
        None
    ));
    assert!(!is_process_matching_provider(
        &unrelated_proc,
        gemini_def,
        None
    ));
}

#[test]
fn test_validate_process_uid_mismatch_rejected() {
    use crate::runtime::linux::ProcessInspection;

    let registry = ProviderRegistry::new();
    let chatgpt_def = registry.get(ProviderId::ChatGPT).unwrap();

    // Mock launcher where get_current_uid returns 1000, but process UID is 0 (root)
    struct UidMismatchLauncher;
    impl std::fmt::Debug for UidMismatchLauncher {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "UidMismatchLauncher")
        }
    }
    impl ProcessLauncher for UidMismatchLauncher {
        fn spawn(&self, _executable: &str, _args: &[String]) -> Result<u32, RuntimeError> {
            Ok(200)
        }
        fn is_running(&self, _pid: u32) -> bool {
            true
        }
        fn inspect_process(&self, pid: u32) -> Result<ProcessInspection, RuntimeError> {
            Ok(ProcessInspection {
                pid,
                exists: true,
                uid: Some(0), // Root UID
                start_time_ticks: Some(555),
                comm: Some("chatgpt-desktop".to_string()),
                cmdline: Some("/snap/bin/chatgpt-desktop".to_string()),
                exe_path: Some("/snap/chatgpt-desktop/current/chatgpt".to_string()),
                state: Some('S'),
                children: Vec::new(),
            })
        }
        fn send_signal(&self, _pid: u32, _signal: i32) -> Result<(), RuntimeError> {
            Ok(())
        }
        fn get_current_uid(&self) -> u32 {
            1000 // Regular user UID
        }
        fn read_meminfo(&self) -> Result<String, RuntimeError> {
            Ok("MemTotal: 4000000 kB\nMemAvailable: 2000000 kB\n".to_string())
        }
    }

    let runtime = LinuxRuntime::with_launcher(Box::new(UidMismatchLauncher));
    let proc = ProcessIdentity {
        pid: 200,
        process_name: "chatgpt-desktop".to_string(),
        start_time_ms: Some(1000),
        start_time_ticks: Some(555),
        executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
        ownership: ProcessOwnership::StartedByAIHub,
    };

    let val_res = runtime.validate_process(&proc, chatgpt_def);
    assert!(
        matches!(val_res, Err(RuntimeError::ProcessVerificationFailed(msg)) if msg.contains("UID"))
    );
}

#[test]
fn test_validate_process_pid_reuse_detected_by_ticks_mismatch() {
    use crate::runtime::linux::ProcessInspection;

    let registry = ProviderRegistry::new();
    let chatgpt_def = registry.get(ProviderId::ChatGPT).unwrap();

    struct PidReuseLauncher;
    impl std::fmt::Debug for PidReuseLauncher {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PidReuseLauncher")
        }
    }
    impl ProcessLauncher for PidReuseLauncher {
        fn spawn(&self, _executable: &str, _args: &[String]) -> Result<u32, RuntimeError> {
            Ok(300)
        }
        fn is_running(&self, _pid: u32) -> bool {
            true
        }
        fn inspect_process(&self, pid: u32) -> Result<ProcessInspection, RuntimeError> {
            Ok(ProcessInspection {
                pid,
                exists: true,
                uid: Some(1000),
                start_time_ticks: Some(99999), // New process start time
                comm: Some("chatgpt-desktop".to_string()),
                cmdline: Some("/snap/bin/chatgpt-desktop".to_string()),
                exe_path: Some("/snap/chatgpt-desktop/current/chatgpt".to_string()),
                state: Some('S'),
                children: Vec::new(),
            })
        }
        fn send_signal(&self, _pid: u32, _signal: i32) -> Result<(), RuntimeError> {
            Ok(())
        }
        fn get_current_uid(&self) -> u32 {
            1000
        }
        fn read_meminfo(&self) -> Result<String, RuntimeError> {
            Ok("MemTotal: 4000000 kB\nMemAvailable: 2000000 kB\n".to_string())
        }
    }

    let runtime = LinuxRuntime::with_launcher(Box::new(PidReuseLauncher));
    let proc = ProcessIdentity {
        pid: 300,
        process_name: "chatgpt-desktop".to_string(),
        start_time_ms: Some(1000),
        start_time_ticks: Some(11111), // Recorded earlier start time
        executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
        ownership: ProcessOwnership::StartedByAIHub,
    };

    let val_res = runtime.validate_process(&proc, chatgpt_def);
    assert!(
        matches!(val_res, Err(RuntimeError::ProcessVerificationFailed(msg)) if msg.contains("PID reuse"))
    );
}

#[test]
fn test_stop_provider_graceful_shutdown_success() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 400,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(12345),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(400);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);

    // Launch first
    manager.launch_provider(ProviderId::ChatGPT).unwrap();
    let state_running = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        state_running.lifecycle_state,
        ApplicationLifecycleState::Running
    );
    assert!(state_running.is_active);

    // Stop provider
    let stopped = manager
        .stop_provider(ProviderId::ChatGPT)
        .expect("Stop must succeed");
    assert_eq!(
        stopped.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
    assert_eq!(stopped.process, None);
    assert!(!stopped.is_active);
    assert_eq!(manager.get_active_provider(), None);
}

#[test]
fn test_stop_provider_rejects_unowned_process() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    // External process launch result
    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 500,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(12345),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::AlreadyRunningExternally, // Not started by AI-Hub
        }),
        details: Some("Spawned externally".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(500);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Attempting to stop an unowned process must fail
    let stop_res = manager.stop_provider(ProviderId::ChatGPT);
    assert!(matches!(
        stop_res,
        Err(RuntimeError::ProcessOwnershipUnknown(_))
    ));
}

#[test]
fn test_discover_all_providers_preserves_running_state_when_alive() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 600,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(12345),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(600);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Verify currently Running
    let state_before_scan = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        state_before_scan.lifecycle_state,
        ApplicationLifecycleState::Running
    );

    // Run discovery scan while process is alive
    let _ = manager.discover_all_providers().unwrap();

    // Running state must NOT be corrupted to Installed!
    let state_after_scan = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        state_after_scan.lifecycle_state,
        ApplicationLifecycleState::Running,
        "System scan must not corrupt verified running application to Installed"
    );
    assert!(state_after_scan.process.is_some());
    assert_eq!(state_after_scan.process.unwrap().pid, 600);
}

#[test]
fn test_discover_all_providers_cleans_up_stale_pid_when_dead() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 700,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(12345),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(700);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Now simulate process termination outside AI-Hub by stopping tracking in runtime
    let _ = manager.stop_provider(ProviderId::ChatGPT).unwrap();

    // Now re-discover
    let _ = manager.discover_all_providers().unwrap();
    let state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(state.lifecycle_state, ApplicationLifecycleState::Installed);
    assert_eq!(state.process, None);
}

#[test]
fn test_get_system_resources_reads_meminfo_and_flags_pressure() {
    use crate::runtime::linux::ProcessInspection;

    struct LowMemLauncher;
    impl std::fmt::Debug for LowMemLauncher {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "LowMemLauncher")
        }
    }
    impl ProcessLauncher for LowMemLauncher {
        fn spawn(&self, _executable: &str, _args: &[String]) -> Result<u32, RuntimeError> {
            Ok(1)
        }
        fn is_running(&self, _pid: u32) -> bool {
            false
        }
        fn inspect_process(&self, _pid: u32) -> Result<ProcessInspection, RuntimeError> {
            Ok(ProcessInspection::default())
        }
        fn send_signal(&self, _pid: u32, _signal: i32) -> Result<(), RuntimeError> {
            Ok(())
        }
        fn get_current_uid(&self) -> u32 {
            1000
        }
        fn read_meminfo(&self) -> Result<String, RuntimeError> {
            // Total 4 GB, only 300 MB available (> 92% used -> should trigger memory pressure warning)
            Ok("\
MemTotal:        4096000 kB
MemFree:          100000 kB
MemAvailable:     307200 kB
Buffers:           50000 kB
Cached:           150000 kB
SwapTotal:       2048000 kB
SwapFree:        1024000 kB
"
            .to_string())
        }
    }

    let runtime = LinuxRuntime::with_launcher(Box::new(LowMemLauncher));
    let snapshot = runtime
        .get_system_resources()
        .expect("Resources must be read");

    assert!(snapshot.is_available);
    assert!(
        snapshot.memory_pressure_warning,
        "Should flag memory pressure when available < 512 MB"
    );
    assert!(snapshot.memory_pressure_message.is_some());
    let sys_mem = snapshot.system_memory.expect("System memory parsed");
    assert_eq!(sys_mem.total_bytes, 4096000 * 1024);
    assert_eq!(sys_mem.available_bytes, 307200 * 1024);
}

#[test]
fn test_parse_stat_state() {
    use crate::runtime::linux::parse_stat_state;

    let stat_running = "1234 (chatgpt-desktop) R 100 100 100 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
    assert_eq!(parse_stat_state(stat_running), Some('R'));

    let stat_sleeping = "1234 (gemini-desktop) S 100 100 100 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
    assert_eq!(parse_stat_state(stat_sleeping), Some('S'));

    let stat_zombie = "1234 (gemini-desktop) Z 100 100 100 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
    assert_eq!(parse_stat_state(stat_zombie), Some('Z'));

    let stat_dead = "1234 (gemini-desktop) X 100 100 100 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
    assert_eq!(parse_stat_state(stat_dead), Some('X'));

    assert_eq!(parse_stat_state("invalid stat"), None);
}

#[test]
fn test_reconcile_provider_states_detects_manual_exit() {
    let app = InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::Gemini,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 800,
            process_name: "gemini-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(12345),
            executable_path: Some("/snap/bin/gemini-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(800);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);
    manager.launch_provider(ProviderId::Gemini).unwrap();

    let state_before = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(
        state_before.lifecycle_state,
        ApplicationLifecycleState::Running
    );
    assert!(state_before.process.is_some());

    // Simulate external manual exit by stopping tracking in runtime
    let _ = manager.stop_provider(ProviderId::Gemini);

    // Call reconcile_provider_states
    let reconciled = manager.reconcile_provider_states().unwrap();
    let gemini_reconciled = reconciled.get(&ProviderId::Gemini).unwrap();
    assert_eq!(
        gemini_reconciled.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
    assert_eq!(gemini_reconciled.process, None);
    assert!(!gemini_reconciled.is_active);
}

#[test]
fn test_stop_provider_when_process_already_exited_reconciles_cleanly() {
    let app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_result = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 850,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(12345),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(850);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(app)),
        Ok(launch_result),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Now simulate process already exited before user clicks Stop
    let stopped = manager.stop_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        stopped.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
    assert_eq!(stopped.process, None);

    // Second stop should return ApplicationNotRunning cleanly since no process is tracked
    let second_stop = manager.stop_provider(ProviderId::ChatGPT);
    assert!(matches!(
        second_stop,
        Err(RuntimeError::ApplicationNotRunning(_))
    ));
}

#[test]
fn test_two_phase_graceful_stop_sigterm_then_sigquit() {
    use crate::runtime::linux::ProcessInspection;

    #[derive(Debug)]
    struct MockElectronLauncher {
        signals_received: Arc<RwLock<Vec<i32>>>,
        is_alive: Arc<RwLock<bool>>,
    }

    impl ProcessLauncher for MockElectronLauncher {
        fn spawn(&self, _executable: &str, _args: &[String]) -> Result<u32, RuntimeError> {
            Ok(900)
        }
        fn is_running(&self, _pid: u32) -> bool {
            *self.is_alive.read().unwrap()
        }
        fn inspect_process(&self, pid: u32) -> Result<ProcessInspection, RuntimeError> {
            Ok(ProcessInspection {
                pid,
                exists: *self.is_alive.read().unwrap(),
                uid: Some(1000),
                start_time_ticks: Some(12345),
                comm: Some("gemini-desktop".to_string()),
                cmdline: Some("/snap/bin/gemini-desktop".to_string()),
                exe_path: Some("/snap/gemini-desktop/current/gemini".to_string()),
                state: Some('S'),
                children: vec![901, 902],
            })
        }
        fn send_signal(&self, _pid: u32, signal: i32) -> Result<(), RuntimeError> {
            self.signals_received.write().unwrap().push(signal);
            if signal == 3 {
                *self.is_alive.write().unwrap() = false;
            }
            Ok(())
        }
        fn get_current_uid(&self) -> u32 {
            1000
        }
        fn read_meminfo(&self) -> Result<String, RuntimeError> {
            Ok("MemTotal: 4000000 kB\nMemAvailable: 2000000 kB\n".to_string())
        }
    }

    let signals = Arc::new(RwLock::new(Vec::new()));
    let is_alive = Arc::new(RwLock::new(true));

    let launcher = Box::new(MockElectronLauncher {
        signals_received: signals.clone(),
        is_alive: is_alive.clone(),
    });

    let runtime = LinuxRuntime::with_launcher(launcher);
    let proc = ProcessIdentity {
        pid: 900,
        process_name: "gemini-desktop".to_string(),
        start_time_ms: Some(1000),
        start_time_ticks: Some(12345),
        executable_path: Some("/snap/bin/gemini-desktop".to_string()),
        ownership: ProcessOwnership::StartedByAIHub,
    };

    let stop_result = runtime.stop_process(&proc, 3, false);
    assert!(
        stop_result.is_ok(),
        "Stop must succeed via graceful signal phase 2"
    );
    assert!(stop_result.unwrap());

    let recorded_signals = signals.read().unwrap().clone();
    assert!(
        recorded_signals.contains(&15),
        "Phase 1 must send SIGTERM (15)"
    );
    assert!(
        recorded_signals.contains(&3),
        "Phase 2 must send SIGQUIT (3) when SIGTERM is intercepted"
    );
    assert!(
        !recorded_signals.contains(&9),
        "Strictly NO force-kill (SIGKILL 9)"
    );
}

#[test]
fn test_uncooperative_process_times_out_truthfully_without_sigkill() {
    use crate::runtime::linux::ProcessInspection;

    #[derive(Debug)]
    struct StubbornLauncher {
        signals_received: Arc<RwLock<Vec<i32>>>,
    }

    impl ProcessLauncher for StubbornLauncher {
        fn spawn(&self, _executable: &str, _args: &[String]) -> Result<u32, RuntimeError> {
            Ok(950)
        }
        fn is_running(&self, _pid: u32) -> bool {
            true // Never stops
        }
        fn inspect_process(&self, pid: u32) -> Result<ProcessInspection, RuntimeError> {
            Ok(ProcessInspection {
                pid,
                exists: true,
                uid: Some(1000),
                start_time_ticks: Some(12345),
                comm: Some("gemini-desktop".to_string()),
                cmdline: Some("/snap/bin/gemini-desktop".to_string()),
                exe_path: Some("/snap/gemini-desktop/current/gemini".to_string()),
                state: Some('S'),
                children: vec![],
            })
        }
        fn send_signal(&self, _pid: u32, signal: i32) -> Result<(), RuntimeError> {
            self.signals_received.write().unwrap().push(signal);
            Ok(())
        }
        fn get_current_uid(&self) -> u32 {
            1000
        }
        fn read_meminfo(&self) -> Result<String, RuntimeError> {
            Ok("MemTotal: 4000000 kB\nMemAvailable: 2000000 kB\n".to_string())
        }
    }

    let signals = Arc::new(RwLock::new(Vec::new()));
    let launcher = Box::new(StubbornLauncher {
        signals_received: signals.clone(),
    });

    let runtime = LinuxRuntime::with_launcher(launcher);
    let proc = ProcessIdentity {
        pid: 950,
        process_name: "gemini-desktop".to_string(),
        start_time_ms: Some(1000),
        start_time_ticks: Some(12345),
        executable_path: Some("/snap/bin/gemini-desktop".to_string()),
        ownership: ProcessOwnership::StartedByAIHub,
    };

    let res = runtime.stop_process(&proc, 3, false);
    assert!(
        matches!(res, Err(RuntimeError::ShutdownTimeout(_))),
        "Must report truthful ShutdownTimeout"
    );

    let recorded = signals.read().unwrap().clone();
    assert!(
        !recorded.contains(&9),
        "Must NEVER deliver SIGKILL 9 under graceful policy"
    );
}

#[test]
fn test_launch_gemini_preserves_chatgpt_running_state() {
    let chatgpt_app = InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    };

    let launch_chatgpt = LaunchResult {
        provider_id: ProviderId::ChatGPT,
        status: LaunchStatus::Running,
        process: Some(ProcessIdentity {
            pid: 1001,
            process_name: "chatgpt-desktop".to_string(),
            start_time_ms: Some(1000),
            start_time_ticks: Some(12345),
            executable_path: Some("/snap/bin/chatgpt-desktop".to_string()),
            ownership: ProcessOwnership::StartedByAIHub,
        }),
        details: Some("Spawned ChatGPT".to_string()),
        launched_at_ms: 1000,
    };

    let mut running_pids = HashSet::new();
    running_pids.insert(1001);

    let runtime = Box::new(MockRuntimePlatform::new(
        Ok(Some(chatgpt_app)),
        Ok(launch_chatgpt),
        running_pids,
    ));

    let manager = AppLifecycleManager::new(runtime);
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    let state_before = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        state_before.lifecycle_state,
        ApplicationLifecycleState::Running
    );

    // Now reconcile provider states - ChatGPT must remain Running
    let reconciled = manager.reconcile_provider_states().unwrap();
    let chatgpt_reconciled = reconciled.get(&ProviderId::ChatGPT).unwrap();
    assert_eq!(
        chatgpt_reconciled.lifecycle_state,
        ApplicationLifecycleState::Running
    );
    assert!(chatgpt_reconciled.process.is_some());
    assert_eq!(chatgpt_reconciled.process.as_ref().unwrap().pid, 1001);
}

// ============================================================================
// MILESTONE 5: SAFE APPLICATION SWITCHING & RESOURCE SAVER ENFORCEMENT TESTS
// ============================================================================

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Barrier;

struct SwitchingMockRuntime {
    installed_apps: Mutex<HashMap<ProviderId, InstalledApplication>>,
    running_pids: Mutex<HashSet<u32>>,
    pid_to_provider: Mutex<HashMap<u32, ProviderId>>,
    next_pid: AtomicU32,
    stop_fail_provider: Mutex<Option<ProviderId>>,
    launch_fail_provider: Mutex<Option<ProviderId>>,
    stopped_providers: Mutex<Vec<ProviderId>>,
    launched_providers: Mutex<Vec<ProviderId>>,
    barrier_on_stop: Mutex<Option<Arc<Barrier>>>,
}

impl SwitchingMockRuntime {
    fn new() -> Self {
        Self {
            installed_apps: Mutex::new(HashMap::new()),
            running_pids: Mutex::new(HashSet::new()),
            pid_to_provider: Mutex::new(HashMap::new()),
            next_pid: AtomicU32::new(100),
            stop_fail_provider: Mutex::new(None),
            launch_fail_provider: Mutex::new(None),
            stopped_providers: Mutex::new(Vec::new()),
            launched_providers: Mutex::new(Vec::new()),
            barrier_on_stop: Mutex::new(None),
        }
    }

    fn register_app(&self, app: InstalledApplication) {
        self.installed_apps
            .lock()
            .unwrap()
            .insert(app.provider_id, app);
    }
}

impl RuntimePlatform for SwitchingMockRuntime {
    fn name(&self) -> &'static str {
        "switching_mock"
    }

    fn detect_application(
        &self,
        provider: &crate::domain::provider::ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        let apps = self.installed_apps.lock().unwrap();
        Ok(apps.get(&provider.id).cloned())
    }

    fn discover_all_applications(
        &self,
        _providers: &[crate::domain::provider::ProviderDefinition],
    ) -> Result<crate::domain::discovery::DiscoverySnapshot, RuntimeError> {
        Ok(crate::domain::discovery::DiscoverySnapshot {
            is_snap_available: true,
            scanned_at_ms: 1000,
            total_packages_scanned: 1,
            results: Vec::new(),
            diagnostics: None,
        })
    }

    fn launch_application(&self, app: &InstalledApplication) -> Result<LaunchResult, RuntimeError> {
        if let Some(fail_id) = self.launch_fail_provider.lock().unwrap().as_ref() {
            if *fail_id == app.provider_id {
                return Err(RuntimeError::LaunchFailed(format!(
                    "Simulated launch failure for {}",
                    app.provider_id.as_str()
                )));
            }
        }

        let pid = self.next_pid.fetch_add(1, Ordering::SeqCst);
        self.running_pids.lock().unwrap().insert(pid);
        self.pid_to_provider
            .lock()
            .unwrap()
            .insert(pid, app.provider_id);
        self.launched_providers
            .lock()
            .unwrap()
            .push(app.provider_id);

        Ok(LaunchResult {
            provider_id: app.provider_id,
            status: LaunchStatus::Running,
            process: Some(ProcessIdentity {
                pid,
                process_name: format!("{}-desktop", app.provider_id.as_str()),
                start_time_ms: Some(1000),
                start_time_ticks: Some(12345),
                executable_path: Some(app.executable_path.clone()),
                ownership: ProcessOwnership::StartedByAIHub,
            }),
            details: Some(format!(
                "Spawned {} with PID {}",
                app.provider_id.as_str(),
                pid
            )),
            launched_at_ms: 1000,
        })
    }

    fn is_process_running(&self, process: &ProcessIdentity) -> Result<bool, RuntimeError> {
        Ok(self.running_pids.lock().unwrap().contains(&process.pid))
    }

    fn validate_process(
        &self,
        process: &ProcessIdentity,
        _provider: &crate::domain::provider::ProviderDefinition,
    ) -> Result<bool, RuntimeError> {
        Ok(self.running_pids.lock().unwrap().contains(&process.pid))
    }

    fn stop_process(
        &self,
        process: &ProcessIdentity,
        _graceful_timeout_secs: u32,
        _allow_force: bool,
    ) -> Result<bool, RuntimeError> {
        if let Some(b) = self.barrier_on_stop.lock().unwrap().clone() {
            b.wait();
        }

        let provider_opt = self
            .pid_to_provider
            .lock()
            .unwrap()
            .get(&process.pid)
            .copied();

        if let Some(provider_id) = provider_opt {
            if let Some(fail_id) = self.stop_fail_provider.lock().unwrap().as_ref() {
                if *fail_id == provider_id {
                    return Err(RuntimeError::ShutdownTimeout(format!(
                        "Graceful shutdown timeout exceeded for PID {}",
                        process.pid
                    )));
                }
            }
            self.stopped_providers.lock().unwrap().push(provider_id);
        }

        self.running_pids.lock().unwrap().remove(&process.pid);
        Ok(true)
    }

    fn get_system_resources(&self) -> Result<crate::domain::ResourceSnapshot, RuntimeError> {
        Ok(crate::domain::ResourceSnapshot {
            is_available: true,
            system_memory: Some(crate::domain::resource::SystemMemory {
                total_bytes: 4 * 1024 * 1024 * 1024,
                available_bytes: 2 * 1024 * 1024 * 1024,
                used_bytes: 2 * 1024 * 1024 * 1024,
                used_percentage: 50,
                swap_total_bytes: 2 * 1024 * 1024 * 1024,
                swap_free_bytes: 1500 * 1024 * 1024,
                swap_used_bytes: 500 * 1024 * 1024,
            }),
            memory_pressure_warning: false,
            memory_pressure_message: None,
            tracked_processes_count: self.running_pids.lock().unwrap().len(),
            process_metrics: Vec::new(),
            diagnostics: Some("Switching mock resources".to_string()),
            timestamp_ms: 1000,
            ..Default::default()
        })
    }
}

#[derive(Clone)]
struct SharedSwitchingMock(Arc<SwitchingMockRuntime>);

impl RuntimePlatform for SharedSwitchingMock {
    fn name(&self) -> &'static str {
        self.0.name()
    }
    fn detect_application(
        &self,
        provider: &crate::domain::provider::ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        self.0.detect_application(provider)
    }
    fn discover_all_applications(
        &self,
        providers: &[crate::domain::provider::ProviderDefinition],
    ) -> Result<crate::domain::discovery::DiscoverySnapshot, RuntimeError> {
        self.0.discover_all_applications(providers)
    }
    fn launch_application(&self, app: &InstalledApplication) -> Result<LaunchResult, RuntimeError> {
        self.0.launch_application(app)
    }
    fn is_process_running(&self, process: &ProcessIdentity) -> Result<bool, RuntimeError> {
        self.0.is_process_running(process)
    }
    fn validate_process(
        &self,
        process: &ProcessIdentity,
        provider: &crate::domain::provider::ProviderDefinition,
    ) -> Result<bool, RuntimeError> {
        self.0.validate_process(process, provider)
    }
    fn stop_process(
        &self,
        process: &ProcessIdentity,
        timeout: u32,
        force: bool,
    ) -> Result<bool, RuntimeError> {
        self.0.stop_process(process, timeout, force)
    }
    fn get_system_resources(&self) -> Result<crate::domain::ResourceSnapshot, RuntimeError> {
        self.0.get_system_resources()
    }
}

#[test]
fn test_milestone_5_successful_provider_switch_with_resource_saver() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));

    // Launch ChatGPT
    let initial_launch = manager.launch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(initial_launch.status, LaunchStatus::Running);
    assert_eq!(manager.get_active_provider(), Some(ProviderId::ChatGPT));

    let chatgpt_state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        chatgpt_state.lifecycle_state,
        ApplicationLifecycleState::Running
    );

    // Switch to Gemini with Resource Saver mode (enabled by default)
    let switched = manager
        .switch_provider(ProviderId::Gemini)
        .expect("Switch must succeed");
    assert_eq!(switched.provider_id, ProviderId::Gemini);
    assert_eq!(switched.lifecycle_state, ApplicationLifecycleState::Running);
    assert!(switched.is_active);
    assert_eq!(manager.get_active_provider(), Some(ProviderId::Gemini));

    // ChatGPT must be stopped and marked Installed
    let chatgpt_after = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        chatgpt_after.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
    assert_eq!(chatgpt_after.process, None);
    assert!(!chatgpt_after.is_active);

    // Verify mock records
    assert_eq!(
        *mock.stopped_providers.lock().unwrap(),
        vec![ProviderId::ChatGPT]
    );
    assert_eq!(
        *mock.launched_providers.lock().unwrap(),
        vec![ProviderId::ChatGPT, ProviderId::Gemini]
    );
}

#[test]
fn test_milestone_5_switch_aborts_on_stop_failure_never_launches_target() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Configure stop failure for ChatGPT
    mock.stop_fail_provider
        .lock()
        .unwrap()
        .replace(ProviderId::ChatGPT);

    let res = manager.switch_provider(ProviderId::Gemini);
    assert!(
        matches!(res, Err(RuntimeError::SwitchFailed(msg)) if msg.contains("Graceful shutdown timeout")),
        "Switch must abort truthfully with SwitchFailed"
    );

    // Target (Gemini) was NEVER launched
    assert_eq!(
        *mock.launched_providers.lock().unwrap(),
        vec![ProviderId::ChatGPT]
    );

    // Target (Gemini) lifecycle state is reverted to Installed, NOT left in Switching
    let gemini_state = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(
        gemini_state.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
}

#[test]
fn test_milestone_5_switch_target_not_installed_fails_before_stopping_previous() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Grok is not installed
    let res = manager.switch_provider(ProviderId::Grok);
    assert!(
        matches!(res, Err(RuntimeError::ApplicationNotInstalled(_))),
        "Must validate target installation before stopping active apps"
    );

    // ChatGPT was NOT stopped
    assert_eq!(mock.stopped_providers.lock().unwrap().len(), 0);
    let chatgpt_state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        chatgpt_state.lifecycle_state,
        ApplicationLifecycleState::Running
    );
}

#[test]
fn test_milestone_5_switch_target_launch_failure_leaves_previous_stopped_and_target_error() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Configure launch failure for Gemini
    mock.launch_fail_provider
        .lock()
        .unwrap()
        .replace(ProviderId::Gemini);

    let res = manager.switch_provider(ProviderId::Gemini);
    assert!(
        matches!(res, Err(RuntimeError::SwitchFailed(msg)) if msg.contains("launch failure")),
        "Switch must report SwitchFailed on target launch error"
    );

    // Previous app (ChatGPT) was successfully stopped
    assert_eq!(
        *mock.stopped_providers.lock().unwrap(),
        vec![ProviderId::ChatGPT]
    );

    // Gemini is marked Error
    let gemini_state = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(
        gemini_state.lifecycle_state,
        ApplicationLifecycleState::Error
    );
    assert!(gemini_state
        .error_message
        .unwrap()
        .contains("launch failure"));

    // ChatGPT remains stopped (Installed)
    let chatgpt_state = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    assert_eq!(
        chatgpt_state.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
}

#[test]
fn test_milestone_5_switch_to_already_running_provider() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Switching to already-running provider activates target and performs no unnecessary stop/start
    let res = manager
        .switch_provider(ProviderId::ChatGPT)
        .expect("Switch to running provider should succeed");
    assert_eq!(res.lifecycle_state, ApplicationLifecycleState::Running);
    assert_eq!(manager.get_active_provider(), Some(ProviderId::ChatGPT));

    assert_eq!(mock.stopped_providers.lock().unwrap().len(), 0);
    assert_eq!(mock.launched_providers.lock().unwrap().len(), 1);
}

#[test]
fn test_milestone_5_multi_instance_when_resource_saver_disabled() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));

    // Disable Resource Saver mode
    let mut settings = manager.get_runtime_settings();
    settings.resource_saver_mode = false;
    manager.update_runtime_settings(settings).unwrap();

    // Launch ChatGPT
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Switch to Gemini with Resource Saver mode OFF
    let switched = manager.switch_provider(ProviderId::Gemini).unwrap();
    assert_eq!(switched.lifecycle_state, ApplicationLifecycleState::Running);

    // ChatGPT was NOT stopped
    assert_eq!(mock.stopped_providers.lock().unwrap().len(), 0);

    // Both providers are currently running
    let chatgpt = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    let gemini = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(chatgpt.lifecycle_state, ApplicationLifecycleState::Running);
    assert_eq!(gemini.lifecycle_state, ApplicationLifecycleState::Running);
}

#[test]
fn test_milestone_5_overlapping_switch_rejected_operation_in_progress() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Claude,
        display_name: "Claude".to_string(),
        version: Some("1.0.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/claude-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("claude-desktop".to_string()),
    });

    let barrier = Arc::new(Barrier::new(2));
    mock.barrier_on_stop
        .lock()
        .unwrap()
        .replace(barrier.clone());

    let manager = Arc::new(AppLifecycleManager::new(Box::new(SharedSwitchingMock(
        mock.clone(),
    ))));
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    let m1 = manager.clone();
    let handle = std::thread::spawn(move || m1.switch_provider(ProviderId::Gemini));

    // Wait until thread 1 enters stop_process and is holding the is_switching lock
    barrier.wait();

    // Now attempt a concurrent switch from another thread
    let concurrent_res = manager.switch_provider(ProviderId::Claude);
    assert!(
        matches!(concurrent_res, Err(RuntimeError::OperationInProgress(_))),
        "Concurrent switch must be immediately rejected with OperationInProgress"
    );

    // Thread 1 completes successfully
    let t1_res = handle.join().expect("Thread 1 must join cleanly");
    assert!(t1_res.is_ok(), "Thread 1 switch should succeed");
}

#[test]
fn test_milestone_5_1_enforced_mode_chatgpt_to_gemini() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    let switched = manager.switch_provider(ProviderId::Gemini).unwrap();
    assert_eq!(switched.lifecycle_state, ApplicationLifecycleState::Running);

    let chatgpt = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    let gemini = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(gemini.lifecycle_state, ApplicationLifecycleState::Running);
    assert_eq!(
        chatgpt.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
    assert!(chatgpt.process.is_none());
}

#[test]
fn test_milestone_5_1_enforced_mode_gemini_to_chatgpt() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    manager.launch_provider(ProviderId::Gemini).unwrap();

    let switched = manager.switch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(switched.lifecycle_state, ApplicationLifecycleState::Running);

    let chatgpt = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    let gemini = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(chatgpt.lifecycle_state, ApplicationLifecycleState::Running);
    assert_eq!(gemini.lifecycle_state, ApplicationLifecycleState::Installed);
    assert!(gemini.process.is_none());
}

#[test]
fn test_milestone_5_1_launching_gemini_while_chatgpt_running_enforces_shutdown_of_chatgpt() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));

    // 1. Launch ChatGPT
    let launch1 = manager.launch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(launch1.status, LaunchStatus::Running);

    // 2. In Enforced mode, invoke ordinary launch_provider for Gemini
    let launch2 = manager.launch_provider(ProviderId::Gemini).unwrap();
    assert_eq!(launch2.status, LaunchStatus::Running);

    // 3. Verify that ChatGPT was stopped and ONLY Gemini is running
    assert_eq!(
        *mock.stopped_providers.lock().unwrap(),
        vec![ProviderId::ChatGPT]
    );

    let chatgpt = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    let gemini = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(
        gemini.lifecycle_state,
        ApplicationLifecycleState::Running,
        "Gemini must be Running"
    );
    assert_eq!(
        chatgpt.lifecycle_state,
        ApplicationLifecycleState::Installed,
        "ChatGPT must be stopped (Installed)"
    );
    assert!(chatgpt.process.is_none(), "ChatGPT process must be cleared");
}

#[test]
fn test_milestone_5_1_launch_aborts_if_stopping_previous_provider_fails() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Configure stop failure for ChatGPT
    mock.stop_fail_provider
        .lock()
        .unwrap()
        .replace(ProviderId::ChatGPT);

    // Launching Gemini must abort because stopping ChatGPT failed
    let launch_res = manager.launch_provider(ProviderId::Gemini);
    assert!(
        launch_res.is_err(),
        "Launch must fail if stopping previous provider fails"
    );

    // Gemini was NEVER launched
    assert_eq!(
        *mock.launched_providers.lock().unwrap(),
        vec![ProviderId::ChatGPT]
    );

    // Gemini state remains Installed, not Running
    let gemini_state = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(
        gemini_state.lifecycle_state,
        ApplicationLifecycleState::Installed
    );
}

#[test]
fn test_milestone_5_1_advisory_mode_permits_both_providers_via_launch() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::Gemini,
        display_name: "Gemini".to_string(),
        version: Some("0.6.0".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/gemini-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("gemini-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));

    // Disable Resource Saver mode (Advisory Only)
    let mut settings = manager.get_runtime_settings();
    settings.resource_saver_mode = false;
    manager.update_runtime_settings(settings).unwrap();

    // Launch ChatGPT
    manager.launch_provider(ProviderId::ChatGPT).unwrap();

    // Launch Gemini
    manager.launch_provider(ProviderId::Gemini).unwrap();

    // Both should be Running and zero stopped providers
    assert_eq!(mock.stopped_providers.lock().unwrap().len(), 0);

    let chatgpt = manager.get_provider_state(ProviderId::ChatGPT).unwrap();
    let gemini = manager.get_provider_state(ProviderId::Gemini).unwrap();
    assert_eq!(chatgpt.lifecycle_state, ApplicationLifecycleState::Running);
    assert_eq!(gemini.lifecycle_state, ApplicationLifecycleState::Running);
}

#[test]
fn test_milestone_5_1_launch_already_running_target_does_not_relaunch() {
    let mock = Arc::new(SwitchingMockRuntime::new());
    mock.register_app(InstalledApplication {
        provider_id: ProviderId::ChatGPT,
        display_name: "ChatGPT".to_string(),
        version: Some("1.0.7".to_string()),
        launch_type: LaunchType::Snap,
        executable_path: "/snap/bin/chatgpt-desktop".to_string(),
        launch_args: vec![],
        icon_path: None,
        package_id: Some("chatgpt-desktop".to_string()),
    });

    let manager = AppLifecycleManager::new(Box::new(SharedSwitchingMock(mock.clone())));
    let res1 = manager.launch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(res1.status, LaunchStatus::Running);

    // Call launch_provider again on the running app
    let res2 = manager.launch_provider(ProviderId::ChatGPT).unwrap();
    assert_eq!(res2.status, LaunchStatus::AlreadyRunning);

    // Spawning should only have happened once
    assert_eq!(mock.launched_providers.lock().unwrap().len(), 1);
}
