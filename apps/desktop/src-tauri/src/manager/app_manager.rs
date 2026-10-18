use crate::{
    domain::{
        discovery::{DiscoverySnapshot, DiscoveryStatus, InstalledApplication},
        error::RuntimeError,
        launch::{LaunchResult, LaunchStatus},
        lifecycle::{ApplicationLifecycleState, ManagedApplication, ProcessOwnership},
        provider::{ProviderDefinition, ProviderId},
        runtime::RuntimeSettings,
        ResourceSnapshot,
    },
    registry::ProviderRegistry,
    runtime::RuntimePlatform,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

pub struct AppLifecycleManager {
    registry: ProviderRegistry,
    runtime: Box<dyn RuntimePlatform>,
    settings: Arc<RwLock<RuntimeSettings>>,
    managed_apps: Arc<RwLock<HashMap<ProviderId, ManagedApplication>>>,
    active_provider: Arc<RwLock<Option<ProviderId>>>,
    is_switching: Arc<AtomicBool>,
}

impl AppLifecycleManager {
    pub fn new(runtime: Box<dyn RuntimePlatform>) -> Self {
        let registry = ProviderRegistry::new();
        let mut initial_apps = HashMap::new();

        // Initialize managed states for all known providers
        for p in registry.list() {
            // Check detection status safely
            let initial_state = match runtime.detect_application(&p) {
                Ok(Some(_)) => ApplicationLifecycleState::Installed,
                Ok(None) => ApplicationLifecycleState::NotInstalled,
                Err(_) => ApplicationLifecycleState::Unknown,
            };

            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            initial_apps.insert(
                p.id,
                ManagedApplication {
                    provider_id: p.id,
                    lifecycle_state: initial_state,
                    process: None,
                    is_active: false,
                    last_state_change_ms: now_ms,
                    error_message: None,
                },
            );
        }

        Self {
            registry,
            runtime,
            settings: Arc::new(RwLock::new(RuntimeSettings::default())),
            managed_apps: Arc::new(RwLock::new(initial_apps)),
            active_provider: Arc::new(RwLock::new(None)),
            is_switching: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn list_providers(&self) -> Vec<ProviderDefinition> {
        self.registry.list()
    }

    pub fn get_provider_definition(
        &self,
        id: ProviderId,
    ) -> Result<ProviderDefinition, RuntimeError> {
        self.registry.get(id).cloned()
    }

    pub fn get_provider_state(&self, id: ProviderId) -> Result<ManagedApplication, RuntimeError> {
        let def = self.registry.get(id)?;
        let mut apps = self.managed_apps.write().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
        })?;

        let managed = apps.get_mut(&id).ok_or_else(|| {
            RuntimeError::ProviderNotFound(format!(
                "Provider {} not found in managed apps",
                id.as_str()
            ))
        })?;

        if managed.lifecycle_state == ApplicationLifecycleState::Running
            || managed.lifecycle_state == ApplicationLifecycleState::Starting
        {
            let pid_alive = if let Some(proc) = &managed.process {
                self.runtime.is_process_running(proc).unwrap_or(false)
            } else {
                false
            };

            let proc_valid = if pid_alive {
                if let Some(proc) = &managed.process {
                    matches!(self.runtime.validate_process(proc, def), Ok(true))
                } else {
                    false
                }
            } else {
                false
            };

            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let is_recent = now_ms.saturating_sub(managed.last_state_change_ms) < 5000;

            if proc_valid {
                if managed.lifecycle_state == ApplicationLifecycleState::Starting {
                    managed.lifecycle_state = ApplicationLifecycleState::Running;
                }
            } else if pid_alive && is_recent {
                // In transient startup grace window: keep current Starting/Running state
            } else {
                managed.process = None;
                managed.is_active = false;
                managed.error_message = None;
                managed.lifecycle_state = ApplicationLifecycleState::Installed;
                managed.last_state_change_ms = now_ms;

                if let Ok(mut active) = self.active_provider.write() {
                    if *active == Some(id) {
                        *active = None;
                    }
                }
            }
        }

        Ok(managed.clone())
    }

    pub fn get_active_provider(&self) -> Option<ProviderId> {
        self.active_provider.read().ok().and_then(|guard| *guard)
    }

    pub fn set_active_provider(&self, id: ProviderId) -> Result<(), RuntimeError> {
        // Validate provider exists
        self.registry.get(id)?;

        let mut active = self.active_provider.write().map_err(|e| {
            RuntimeError::Internal(format!(
                "Failed to acquire write lock on active provider: {e}"
            ))
        })?;
        *active = Some(id);

        // Update is_active flags across all managed applications
        let mut apps = self.managed_apps.write().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
        })?;

        for (pid, app) in apps.iter_mut() {
            app.is_active = *pid == id;
        }

        Ok(())
    }

    pub fn get_runtime_settings(&self) -> RuntimeSettings {
        self.settings.read().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn update_runtime_settings(
        &self,
        new_settings: RuntimeSettings,
    ) -> Result<RuntimeSettings, RuntimeError> {
        new_settings.validate()?;

        let mut settings = self.settings.write().map_err(|e| {
            RuntimeError::Internal(format!(
                "Failed to acquire write lock on runtime settings: {e}"
            ))
        })?;
        *settings = new_settings.clone();
        Ok(new_settings)
    }

    pub fn discover_provider(
        &self,
        id: ProviderId,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        let def = self.registry.get(id)?;
        let app = self.runtime.detect_application(def)?;
        let is_installed = app.is_some();

        // Update managed state with discovered reality
        let mut apps = self.managed_apps.write().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
        })?;

        if let Some(managed) = apps.get_mut(&id) {
            let is_still_running = if let Some(proc) = &managed.process {
                self.runtime.is_process_running(proc).unwrap_or(false)
            } else {
                false
            };

            if is_still_running
                && (managed.lifecycle_state == ApplicationLifecycleState::Running
                    || managed.lifecycle_state == ApplicationLifecycleState::Starting)
            {
                // Process is alive; preserve Running state and process identity.
            } else {
                if managed.process.is_some() && !is_still_running {
                    managed.process = None;
                }
                managed.lifecycle_state = if is_installed {
                    ApplicationLifecycleState::Installed
                } else {
                    ApplicationLifecycleState::NotInstalled
                };
                managed.last_state_change_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
            }
        }

        Ok(app)
    }

    pub fn discover_all_providers(&self) -> Result<DiscoverySnapshot, RuntimeError> {
        let providers = self.registry.list();
        let snapshot = self.runtime.discover_all_applications(&providers)?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut apps = self.managed_apps.write().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
        })?;

        for res in &snapshot.results {
            if let Some(managed) = apps.get_mut(&res.provider_id) {
                let is_still_running = if let Some(proc) = &managed.process {
                    self.runtime.is_process_running(proc).unwrap_or(false)
                } else {
                    false
                };

                if is_still_running
                    && (managed.lifecycle_state == ApplicationLifecycleState::Running
                        || managed.lifecycle_state == ApplicationLifecycleState::Starting)
                {
                    // Preserve running state and process identity
                    continue;
                }

                if managed.process.is_some() && !is_still_running {
                    managed.process = None;
                }

                match res.status {
                    DiscoveryStatus::Confirmed => {
                        managed.lifecycle_state = ApplicationLifecycleState::Installed;
                        managed.last_state_change_ms = now_ms;
                        managed.error_message = None;
                    }
                    DiscoveryStatus::NotInstalled => {
                        managed.lifecycle_state = ApplicationLifecycleState::NotInstalled;
                        managed.last_state_change_ms = now_ms;
                        managed.error_message = None;
                    }
                    DiscoveryStatus::Ambiguous => {
                        managed.lifecycle_state = ApplicationLifecycleState::Unknown;
                        managed.last_state_change_ms = now_ms;
                        managed.error_message = res.details.clone();
                    }
                    DiscoveryStatus::Unavailable | DiscoveryStatus::Failed => {
                        managed.error_message = res.details.clone();
                    }
                }
            }
        }

        Ok(snapshot)
    }

    pub fn launch_provider(&self, id: ProviderId) -> Result<LaunchResult, RuntimeError> {
        let def = self.registry.get(id)?;
        let settings = self.get_runtime_settings();

        // 1. Check if already running with active process
        {
            let apps = self.managed_apps.read().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire read lock on managed apps: {e}"))
            })?;
            if let Some(managed) = apps.get(&id) {
                if managed.lifecycle_state == ApplicationLifecycleState::Running {
                    if let Some(proc) = &managed.process {
                        if self.runtime.is_process_running(proc).unwrap_or(false) {
                            return Ok(LaunchResult {
                                provider_id: id,
                                status: LaunchStatus::AlreadyRunning,
                                process: Some(proc.clone()),
                                details: Some(format!(
                                    "Application {} is already running with PID {}",
                                    id.as_str(),
                                    proc.pid
                                )),
                                launched_at_ms: proc
                                    .start_time_ms
                                    .unwrap_or(managed.last_state_change_ms),
                            });
                        }
                    }
                }
            }
        }

        // 2. Reject if another switch or launch operation is already in progress
        if self.is_switching.load(Ordering::SeqCst) {
            return Err(RuntimeError::OperationInProgress(
                "A provider switch or launch is already in progress".to_string(),
            ));
        }

        // 3. Check if other providers are currently running
        let other_running = {
            let apps = self.managed_apps.read().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire read lock on managed apps: {e}"))
            })?;
            apps.iter().any(|(&pid, app)| {
                pid != id
                    && (app.lifecycle_state == ApplicationLifecycleState::Running
                        || app.lifecycle_state == ApplicationLifecycleState::Starting)
                    && app.process.is_some()
            })
        };

        // 4. In Resource Saver (Enforced) mode, if another application is running,
        // apply the safe serialized switching policy (gracefully stop old app and verify exit before launching target)
        if settings.resource_saver_mode && other_running {
            let switched = self.switch_provider(id)?;
            return Ok(LaunchResult {
                provider_id: id,
                status: LaunchStatus::Running,
                process: switched.process,
                details: Some(format!(
                    "Application {} launched with Resource Saver single-instance enforcement (switched from previous provider)",
                    def.name
                )),
                launched_at_ms: switched.last_state_change_ms,
            });
        }

        // 5. In Advisory Only mode (or when no other app is running), launch directly
        self.launch_provider_internal(id)
    }

    pub(crate) fn launch_provider_internal(
        &self,
        id: ProviderId,
    ) -> Result<LaunchResult, RuntimeError> {
        let def = self.registry.get(id)?;
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Stage 1: State validation and in-flight guard
        {
            let mut apps = self.managed_apps.write().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
            })?;

            let managed = apps.get_mut(&id).ok_or_else(|| {
                RuntimeError::ProviderNotFound(format!(
                    "Provider {} not found in managed apps",
                    id.as_str()
                ))
            })?;

            // Guard against duplicate in-flight requests
            if managed.lifecycle_state == ApplicationLifecycleState::Starting {
                return Err(RuntimeError::OperationInProgress(format!(
                    "Launch operation already in progress for {}",
                    id.as_str()
                )));
            }

            // Check if already running with active process
            if managed.lifecycle_state == ApplicationLifecycleState::Running {
                if let Some(proc) = &managed.process {
                    if self.runtime.is_process_running(proc)? {
                        return Ok(LaunchResult {
                            provider_id: id,
                            status: LaunchStatus::AlreadyRunning,
                            process: Some(proc.clone()),
                            details: Some(format!(
                                "Application {} is already running with PID {}",
                                id.as_str(),
                                proc.pid
                            )),
                            launched_at_ms: proc.start_time_ms.unwrap_or(now_ms),
                        });
                    }
                }
                // Process is dead / stale: clear process identity and reset state
                managed.process = None;
                managed.lifecycle_state = ApplicationLifecycleState::Installed;
            }

            // Mark Starting to lock out concurrent attempts
            managed.lifecycle_state = ApplicationLifecycleState::Starting;
            managed.last_state_change_ms = now_ms;
            managed.error_message = None;
        }

        // Stage 2: Application detection
        let app_result = self.runtime.detect_application(def);
        let app = match app_result {
            Ok(Some(app)) => app,
            Ok(None) => {
                if let Ok(mut apps) = self.managed_apps.write() {
                    if let Some(managed) = apps.get_mut(&id) {
                        managed.lifecycle_state = ApplicationLifecycleState::NotInstalled;
                        managed.error_message = Some("Application is not installed".to_string());
                        managed.last_state_change_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                    }
                }
                return Err(RuntimeError::ApplicationNotInstalled(format!(
                    "Application for provider {} is not installed on this system",
                    id.as_str()
                )));
            }
            Err(e) => {
                if let Ok(mut apps) = self.managed_apps.write() {
                    if let Some(managed) = apps.get_mut(&id) {
                        managed.lifecycle_state = ApplicationLifecycleState::Error;
                        managed.error_message = Some(e.to_string());
                        managed.last_state_change_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                    }
                }
                return Err(e);
            }
        };

        // Stage 3: Execution through runtime platform
        match self.runtime.launch_application(&app) {
            Ok(result) => {
                // Update managed application record
                let mut apps = self.managed_apps.write().map_err(|e| {
                    RuntimeError::Internal(format!(
                        "Failed to acquire write lock on managed apps: {e}"
                    ))
                })?;

                if let Some(managed) = apps.get_mut(&id) {
                    managed.process = result.process.clone();
                    managed.lifecycle_state = if result.status == LaunchStatus::Running {
                        ApplicationLifecycleState::Running
                    } else {
                        ApplicationLifecycleState::Starting
                    };
                    managed.last_state_change_ms = result.launched_at_ms;
                    managed.error_message = None;
                }

                // Set active provider
                if let Ok(mut active) = self.active_provider.write() {
                    *active = Some(id);
                }
                for (pid, a) in apps.iter_mut() {
                    a.is_active = *pid == id;
                }

                Ok(result)
            }
            Err(err) => {
                if let Ok(mut apps) = self.managed_apps.write() {
                    if let Some(managed) = apps.get_mut(&id) {
                        managed.lifecycle_state = ApplicationLifecycleState::Error;
                        managed.error_message = Some(err.to_string());
                        managed.last_state_change_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                    }
                }
                Err(err)
            }
        }
    }

    pub fn stop_provider(&self, id: ProviderId) -> Result<ManagedApplication, RuntimeError> {
        let def = self.registry.get(id)?;
        let settings = self.get_runtime_settings();
        let timeout_secs = settings.graceful_shutdown_timeout_secs;
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // 1. Inspect state and extract process identity
        let (proc_identity, current_state) = {
            let apps = self.managed_apps.read().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire read lock on managed apps: {e}"))
            })?;
            let managed = apps.get(&id).ok_or_else(|| {
                RuntimeError::ProviderNotFound(format!(
                    "Provider {} not found in managed apps",
                    id.as_str()
                ))
            })?;
            (managed.process.clone(), managed.lifecycle_state)
        };

        let proc = match proc_identity {
            Some(p) => p,
            None => {
                return Err(RuntimeError::ApplicationNotRunning(format!(
                    "Cannot stop provider {}: no running process tracked",
                    id.as_str()
                )));
            }
        };

        if current_state != ApplicationLifecycleState::Running
            && current_state != ApplicationLifecycleState::Starting
        {
            return Err(RuntimeError::ApplicationNotRunning(format!(
                "Cannot stop provider {}: lifecycle state is {:?}",
                id.as_str(),
                current_state
            )));
        }

        // 2. Ownership check: must be StartedByAIHub
        if proc.ownership != ProcessOwnership::StartedByAIHub {
            return Err(RuntimeError::ProcessOwnershipUnknown(format!(
                "Cannot stop provider {}: process PID {} was not started by AI-Hub",
                id.as_str(),
                proc.pid
            )));
        }

        // 3. Check if process has already exited
        let is_running = self.runtime.is_process_running(&proc).unwrap_or(false);
        if !is_running {
            let mut apps = self.managed_apps.write().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
            })?;
            if let Some(managed) = apps.get_mut(&id) {
                managed.lifecycle_state = ApplicationLifecycleState::Installed;
                managed.process = None;
                managed.is_active = false;
                managed.error_message = None;
                managed.last_state_change_ms = now_ms;
            }
            if let Ok(mut active) = self.active_provider.write() {
                if *active == Some(id) {
                    *active = None;
                }
            }
            return apps.get(&id).cloned().ok_or_else(|| {
                RuntimeError::ProviderNotFound(format!("Provider {} not found", id.as_str()))
            });
        }

        // 4. Process identity validation before touching the process
        match self.runtime.validate_process(&proc, def) {
            Ok(true) => {}
            Ok(false) => {
                let mut apps = self.managed_apps.write().map_err(|e| {
                    RuntimeError::Internal(format!(
                        "Failed to acquire write lock on managed apps: {e}"
                    ))
                })?;
                if let Some(managed) = apps.get_mut(&id) {
                    managed.lifecycle_state = ApplicationLifecycleState::Error;
                    managed.error_message = Some(format!(
                        "Process verification failed for PID {}: does not match provider {}",
                        proc.pid,
                        id.as_str()
                    ));
                    managed.last_state_change_ms = now_ms;
                }
                return Err(RuntimeError::ProcessVerificationFailed(format!(
                    "Process verification failed for PID {}: does not match provider {}",
                    proc.pid,
                    id.as_str()
                )));
            }
            Err(e) => {
                let mut apps = self.managed_apps.write().map_err(|err| {
                    RuntimeError::Internal(format!(
                        "Failed to acquire write lock on managed apps: {err}"
                    ))
                })?;
                if let Some(managed) = apps.get_mut(&id) {
                    managed.lifecycle_state = ApplicationLifecycleState::Error;
                    managed.error_message = Some(e.to_string());
                    managed.last_state_change_ms = now_ms;
                }
                return Err(e);
            }
        }

        // 5. Mark Stopping state
        {
            let mut apps = self.managed_apps.write().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
            })?;
            if let Some(managed) = apps.get_mut(&id) {
                managed.lifecycle_state = ApplicationLifecycleState::Stopping;
                managed.last_state_change_ms = now_ms;
            }
        }

        // 6. Send SIGTERM / SIGQUIT and await exit within timeout (strictly no SIGKILL)
        match self.runtime.stop_process(&proc, timeout_secs, false) {
            Ok(_) => {
                let mut apps = self.managed_apps.write().map_err(|e| {
                    RuntimeError::Internal(format!(
                        "Failed to acquire write lock on managed apps: {e}"
                    ))
                })?;

                let managed = apps.get_mut(&id).ok_or_else(|| {
                    RuntimeError::ProviderNotFound(format!("Provider {} not found", id.as_str()))
                })?;

                managed.lifecycle_state = ApplicationLifecycleState::Installed;
                managed.process = None;
                managed.is_active = false;
                managed.error_message = None;
                managed.last_state_change_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                if let Ok(mut active) = self.active_provider.write() {
                    if *active == Some(id) {
                        *active = None;
                    }
                }

                Ok(managed.clone())
            }
            Err(err) => {
                let mut apps = self.managed_apps.write().map_err(|e| {
                    RuntimeError::Internal(format!(
                        "Failed to acquire write lock on managed apps: {e}"
                    ))
                })?;

                if let Some(managed) = apps.get_mut(&id) {
                    managed.lifecycle_state = ApplicationLifecycleState::Error;
                    managed.error_message = Some(err.to_string());
                    managed.last_state_change_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                }

                Err(err)
            }
        }
    }

    pub fn reconcile_provider_states(
        &self,
    ) -> Result<HashMap<ProviderId, ManagedApplication>, RuntimeError> {
        let providers = self.registry.list();
        let mut apps = self.managed_apps.write().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
        })?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        for p in &providers {
            if let Some(managed) = apps.get_mut(&p.id) {
                // If this provider is currently switching, do not disrupt it
                if managed.lifecycle_state == ApplicationLifecycleState::Switching {
                    continue;
                }

                if managed.lifecycle_state == ApplicationLifecycleState::Running
                    || managed.lifecycle_state == ApplicationLifecycleState::Starting
                {
                    let pid_alive = if let Some(proc) = &managed.process {
                        self.runtime.is_process_running(proc).unwrap_or(false)
                    } else {
                        false
                    };

                    let proc_valid = if pid_alive {
                        if let Some(proc) = &managed.process {
                            matches!(self.runtime.validate_process(proc, p), Ok(true))
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let is_recent = now_ms.saturating_sub(managed.last_state_change_ms) < 5000;

                    if proc_valid {
                        if managed.lifecycle_state == ApplicationLifecycleState::Starting {
                            managed.lifecycle_state = ApplicationLifecycleState::Running;
                        }
                    } else if pid_alive && is_recent {
                        // Transient startup grace window: keep current Starting/Running state
                    } else {
                        managed.process = None;
                        managed.is_active = false;
                        managed.error_message = None;
                        managed.lifecycle_state = ApplicationLifecycleState::Installed;
                        managed.last_state_change_ms = now_ms;

                        if let Ok(mut active) = self.active_provider.write() {
                            if *active == Some(p.id) {
                                *active = None;
                            }
                        }
                    }
                }
            }
        }

        Ok(apps.clone())
    }

    pub fn get_system_resources(&self) -> Result<ResourceSnapshot, RuntimeError> {
        let mut snapshot = self.runtime.get_system_resources()?;

        if let Ok(apps) = self.managed_apps.read() {
            let running_count = apps
                .values()
                .filter(|a| a.lifecycle_state == ApplicationLifecycleState::Running)
                .count();

            if running_count > 0 {
                let diag = match snapshot.diagnostics {
                    Some(d) => format!("{d} ({running_count} provider(s) active)"),
                    None => format!("{running_count} provider(s) active"),
                };
                snapshot.diagnostics = Some(diag);
            }
        }

        Ok(snapshot)
    }

    fn set_target_installed_or_error(
        &self,
        id: ProviderId,
        error: Option<String>,
    ) -> Result<(), RuntimeError> {
        let mut apps = self.managed_apps.write().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
        })?;
        if let Some(target) = apps.get_mut(&id) {
            target.lifecycle_state = if error.is_some() {
                ApplicationLifecycleState::Error
            } else {
                ApplicationLifecycleState::Installed
            };
            target.error_message = error;
            target.last_state_change_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
        }
        Ok(())
    }

    pub fn switch_provider(&self, id: ProviderId) -> Result<ManagedApplication, RuntimeError> {
        let def = self.registry.get(id)?;

        // Enforce concurrency guard against simultaneous or overlapping switch requests
        if self
            .is_switching
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(RuntimeError::OperationInProgress(
                "Another provider switch is already in progress".to_string(),
            ));
        }

        struct SwitchingGuard<'a>(&'a AtomicBool);
        impl<'a> Drop for SwitchingGuard<'a> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::SeqCst);
            }
        }
        let _guard = SwitchingGuard(&self.is_switching);

        let settings = self.get_runtime_settings();

        // 1. Check if target is already running
        let is_target_already_running = {
            let apps = self.managed_apps.read().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire read lock on managed apps: {e}"))
            })?;
            apps.get(&id)
                .map(|t| {
                    t.lifecycle_state == ApplicationLifecycleState::Running && t.process.is_some()
                })
                .unwrap_or(false)
        };

        // 2. Target installation and eligibility verification
        // Target must be installed before we even consider stopping any running application!
        let is_installed = match self.runtime.detect_application(def) {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(RuntimeError::AmbiguousMatch(msg)) => {
                return Err(RuntimeError::AmbiguousMatch(msg))
            }
            Err(e) => return Err(e),
        };

        if !is_installed {
            return Err(RuntimeError::ApplicationNotInstalled(format!(
                "Cannot switch to {}: target application is not installed on this system",
                def.name
            )));
        }

        // 3. Identify running applications that require shutdown
        // In Resource Saver mode, we must stop other running managed applications.
        // If Resource Saver mode is disabled, multiple applications may run concurrently.
        let providers_to_stop: Vec<ProviderId> = if settings.resource_saver_mode {
            let apps = self.managed_apps.read().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire read lock on managed apps: {e}"))
            })?;
            apps.iter()
                .filter(|(&pid, app)| {
                    pid != id
                        && (app.lifecycle_state == ApplicationLifecycleState::Running
                            || app.lifecycle_state == ApplicationLifecycleState::Starting)
                        && app.process.is_some()
                })
                .map(|(&pid, _)| pid)
                .collect()
        } else {
            Vec::new()
        };

        // If target is already running:
        if is_target_already_running {
            // Under Resource Saver mode, stop any other running providers
            for old_id in providers_to_stop {
                self.stop_provider(old_id).map_err(|e| {
                    RuntimeError::SwitchFailed(format!(
                        "Failed to stop previous provider {}: {e}",
                        old_id.as_str()
                    ))
                })?;
            }

            self.set_active_provider(id)?;
            let apps = self
                .managed_apps
                .read()
                .map_err(|e| RuntimeError::Internal(format!("Failed to acquire read lock: {e}")))?;
            return apps.get(&id).cloned().ok_or_else(|| {
                RuntimeError::ProviderNotFound(format!("Provider {} not found", id.as_str()))
            });
        }

        // 4. Mark target state as Switching to show transition in progress
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        {
            let mut apps = self.managed_apps.write().map_err(|e| {
                RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
            })?;
            if let Some(target) = apps.get_mut(&id) {
                target.lifecycle_state = ApplicationLifecycleState::Switching;
                target.last_state_change_ms = now_ms;
                target.error_message = None;
            }
        }

        // 5. Gracefully stop each running provider and verify its termination
        for old_id in providers_to_stop {
            match self.stop_provider(old_id) {
                Ok(stopped_app) => {
                    // Double-check verification: process must be None and state must be Installed
                    if stopped_app.process.is_some()
                        || stopped_app.lifecycle_state != ApplicationLifecycleState::Installed
                    {
                        let _ = self.set_target_installed_or_error(id, None);
                        return Err(RuntimeError::SwitchFailed(format!(
                            "Failed to verify shutdown of previous provider {}",
                            old_id.as_str()
                        )));
                    }
                }
                Err(err) => {
                    // Abort switch! Old provider refused to shut down or timed out.
                    // Target must NOT be launched and should revert to Installed.
                    let _ = self.set_target_installed_or_error(id, None);
                    return Err(RuntimeError::SwitchFailed(format!(
                        "Switch aborted: previous provider {} failed to stop: {err}",
                        old_id.as_str()
                    )));
                }
            }
        }

        // 6. Launch the target provider only after previous provider has verified stopped
        match self.launch_provider_internal(id) {
            Ok(_launch_result) => {
                let apps = self.managed_apps.read().map_err(|e| {
                    RuntimeError::Internal(format!(
                        "Failed to acquire read lock on managed apps: {e}"
                    ))
                })?;
                let target_app = apps.get(&id).cloned().ok_or_else(|| {
                    RuntimeError::ProviderNotFound(format!("Provider {} not found", id.as_str()))
                })?;
                Ok(target_app)
            }
            Err(err) => {
                // Target failed to launch after old provider stopped.
                // Accurately reflect that target is in Error state, and return SwitchFailed.
                Err(RuntimeError::SwitchFailed(format!(
                    "Previous application stopped, but target provider {} failed to launch: {err}",
                    id.as_str()
                )))
            }
        }
    }
}
