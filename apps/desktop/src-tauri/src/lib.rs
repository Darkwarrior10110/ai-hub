pub mod commands;
pub mod domain;
pub mod manager;
pub mod registry;
pub mod runtime;

#[cfg(test)]
mod tests;

use manager::AppLifecycleManager;
use runtime::create_default_runtime;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let lifecycle_manager = Arc::new(AppLifecycleManager::new(create_default_runtime()));

    tauri::Builder::default()
        .manage(lifecycle_manager)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_providers,
            commands::get_provider_state,
            commands::get_active_provider,
            commands::set_active_provider,
            commands::get_runtime_settings,
            commands::update_runtime_settings,
            commands::discover_provider,
            commands::launch_provider,
            commands::stop_provider,
            commands::switch_provider,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
