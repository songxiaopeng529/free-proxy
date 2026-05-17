use tauri::State;

use crate::core::system_proxy as sys;
use crate::model::app_config::AppState;
use crate::model::proxy::SystemProxyStatus;

#[tauri::command]
pub fn enable_system_proxy(state: State<'_, AppState>) -> Result<(), String> {
    let port = state.config.lock().unwrap().mixed_port;
    sys::enable_system_proxy(port)
}

#[tauri::command]
pub fn disable_system_proxy() -> Result<(), String> {
    sys::disable_system_proxy()
}

#[tauri::command]
pub fn get_system_proxy_status() -> Result<SystemProxyStatus, String> {
    sys::get_system_proxy_status()
}
