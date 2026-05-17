use tauri::State;

use crate::core::config_generator;
use crate::model::app_config::AppState;
use crate::model::proxy::MihomoStatus;
use crate::utils::paths;

#[tauri::command]
pub fn start_proxy(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mihomo_dir = paths::mihomo_dir(&app);
    let config = state.config.lock().unwrap().clone();
    let subs = state.subscriptions.lock().unwrap().clone();

    let yaml = config_generator::generate_config(&config, &subs)?;
    std::fs::write(mihomo_dir.join("config.yaml"), yaml)
        .map_err(|e| format!("write config error: {}", e))?;

    state.mihomo.start(&app, &mihomo_dir)?;

    // Wait briefly for mihomo to start
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

#[tauri::command]
pub fn stop_proxy(state: State<'_, AppState>) -> Result<(), String> {
    state.mihomo.stop()
}

#[tauri::command]
pub fn restart_proxy(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mihomo_dir = paths::mihomo_dir(&app);
    let config = state.config.lock().unwrap().clone();
    let subs = state.subscriptions.lock().unwrap().clone();

    let yaml = config_generator::generate_config(&config, &subs)?;
    std::fs::write(mihomo_dir.join("config.yaml"), yaml)
        .map_err(|e| format!("write config error: {}", e))?;

    state.mihomo.restart(&app, &mihomo_dir)
}

#[tauri::command]
pub async fn get_proxy_status(state: State<'_, AppState>) -> Result<MihomoStatus, String> {
    Ok(state.mihomo.status().await)
}
