use tauri::State;

use crate::model::app_config::AppState;
use crate::model::rule::DEFAULT_RULES;
use crate::utils::paths;

#[tauri::command]
pub fn get_rules(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state.config.lock().unwrap();
    if config.custom_rules.is_empty() {
        Ok(DEFAULT_RULES.iter().map(|s| s.to_string()).collect())
    } else {
        Ok(config.custom_rules.clone())
    }
}

#[tauri::command]
pub fn set_custom_rules(
    rules: Vec<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut config = state.config.lock().unwrap();
        config.custom_rules = rules;
        paths::save_app_config(&app, &config);
    }

    if state.mihomo.is_running() {
        crate::cmd::proxy::restart_proxy(app, state)?;
    }

    Ok(())
}
