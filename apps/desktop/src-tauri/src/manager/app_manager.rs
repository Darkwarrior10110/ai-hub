use crate::{
    domain::{
        discovery::{DiscoverySnapshot, DiscoveryStatus, InstalledApplication},
        error::RuntimeError,
        lifecycle::{ApplicationLifecycleState, ManagedApplication},
        provider::{ProviderDefinition, ProviderId},
        runtime::RuntimeSettings,
    },
    registry::ProviderRegistry,
    runtime::RuntimePlatform,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct AppLifecycleManager {
    registry: ProviderRegistry,
    runtime: Box<dyn RuntimePlatform>,
    settings: Arc<RwLock<RuntimeSettings>>,
    managed_apps: Arc<RwLock<HashMap<ProviderId, ManagedApplication>>>,
    active_provider: Arc<RwLock<Option<ProviderId>>>,
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
        let apps = self.managed_apps.read().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire read lock on managed apps: {e}"))
        })?;

        apps.get(&id).cloned().ok_or_else(|| {
            RuntimeError::ProviderNotFound(format!(
                "Provider {} not found in managed apps",
                id.as_str()
            ))
        })
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

        // Update managed state with discovered reality
        let mut apps = self.managed_apps.write().map_err(|e| {
            RuntimeError::Internal(format!("Failed to acquire write lock on managed apps: {e}"))
        })?;

        if let Some(managed) = apps.get_mut(&id) {
            managed.lifecycle_state = if app.is_some() {
                ApplicationLifecycleState::Installed
            } else {
                ApplicationLifecycleState::NotInstalled
            };
            managed.last_state_change_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
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

    pub fn launch_provider(&self, id: ProviderId) -> Result<ManagedApplication, RuntimeError> {
        self.registry.get(id)?;
        // Explicitly return structured error for Milestone 1 as real process execution is scoped for Milestone 3
        Err(RuntimeError::NotImplemented(format!(
            "Process execution for {} will be activated in Milestone 3",
            id.as_str()
        )))
    }

    pub fn stop_provider(&self, id: ProviderId) -> Result<ManagedApplication, RuntimeError> {
        self.registry.get(id)?;
        Err(RuntimeError::NotImplemented(format!(
            "Process shutdown for {} will be activated in Milestone 4",
            id.as_str()
        )))
    }

    pub fn switch_provider(&self, id: ProviderId) -> Result<ManagedApplication, RuntimeError> {
        self.registry.get(id)?;
        Err(RuntimeError::NotImplemented(format!(
            "Provider switching to {} will be activated in Milestone 5",
            id.as_str()
        )))
    }
}
