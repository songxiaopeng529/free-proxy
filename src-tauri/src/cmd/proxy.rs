use tauri::State;

use crate::core::config_generator;
use crate::model::app_config::AppState;
use crate::model::proxy::MihomoStatus;
use crate::utils::paths;

#[tauri::command]
pub async fn start_proxy(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mihomo_dir = paths::mihomo_dir(&app);
    let config = state.config.lock().unwrap().clone();
    let subs = state.subscriptions.lock().unwrap().clone();

    let yaml = config_generator::generate_config(&config, &subs)?;
    std::fs::write(mihomo_dir.join("config.yaml"), yaml)
        .map_err(|e| format!("write config error: {}", e))?;

    state.mihomo.start(&app, &mihomo_dir)?;

    for _ in 0..20 {
        if state.mihomo.health_check().await {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    state.mihomo.stop().ok();
    Err("mihomo started but controller did not become ready on 127.0.0.1:9090".to_string())
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
