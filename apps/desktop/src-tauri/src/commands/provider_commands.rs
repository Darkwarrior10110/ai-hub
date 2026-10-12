use crate::{
    domain::{
        discovery::{DiscoverySnapshot, InstalledApplication},
        error::RuntimeError,
        lifecycle::ManagedApplication,
        provider::{ProviderDefinition, ProviderId},
        runtime::RuntimeSettings,
    },
    manager::AppLifecycleManager,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn list_providers(
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<Vec<ProviderDefinition>, String> {
    Ok(manager.list_providers())
}

#[tauri::command]
pub fn get_provider_state(
    provider_id: String,
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<ManagedApplication, String> {
    let id: ProviderId = provider_id
        .parse()
        .map_err(|e: RuntimeError| e.to_string())?;
    manager.get_provider_state(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_provider(
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<Option<String>, String> {
    Ok(manager
        .get_active_provider()
        .map(|id| id.as_str().to_string()))
}

#[tauri::command]
pub fn set_active_provider(
    provider_id: String,
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<(), String> {
    let id: ProviderId = provider_id
        .parse()
        .map_err(|e: RuntimeError| e.to_string())?;
    manager.set_active_provider(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_runtime_settings(
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<RuntimeSettings, String> {
    Ok(manager.get_runtime_settings())
}

#[tauri::command]
pub fn update_runtime_settings(
    settings: RuntimeSettings,
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<RuntimeSettings, String> {
    manager
        .update_runtime_settings(settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn discover_provider(
    provider_id: String,
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<Option<InstalledApplication>, String> {
    let id: ProviderId = provider_id
        .parse()
        .map_err(|e: RuntimeError| e.to_string())?;
    manager.discover_provider(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn discover_all_providers(
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<DiscoverySnapshot, String> {
    manager.discover_all_providers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn launch_provider(
    provider_id: String,
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<ManagedApplication, String> {
    let id: ProviderId = provider_id
        .parse()
        .map_err(|e: RuntimeError| e.to_string())?;
    manager.launch_provider(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_provider(
    provider_id: String,
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<ManagedApplication, String> {
    let id: ProviderId = provider_id
        .parse()
        .map_err(|e: RuntimeError| e.to_string())?;
    manager.stop_provider(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn switch_provider(
    provider_id: String,
    manager: State<Arc<AppLifecycleManager>>,
) -> Result<ManagedApplication, String> {
    let id: ProviderId = provider_id
        .parse()
        .map_err(|e: RuntimeError| e.to_string())?;
    manager.switch_provider(id).map_err(|e| e.to_string())
}
