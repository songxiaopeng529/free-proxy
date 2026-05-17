use tauri::State;

use crate::model::app_config::{AppConfig, AppState};
use crate::model::proxy::ProxyGroup;
use crate::utils::paths;

#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub async fn set_proxy_mode(
    mode: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut config = state.config.lock().unwrap();
        config.mode = mode.clone();
        paths::save_app_config(&app, &config);
    }

    if state.mihomo.is_running() {
        let port = state.config.lock().unwrap().controller_port;
        let secret = state.config.lock().unwrap().secret.clone();
        let client = reqwest::Client::new();
        let mut req = client.patch(format!("http://127.0.0.1:{}/configs", port));
        if !secret.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", secret));
        }
        req.json(&serde_json::json!({"mode": mode}))
            .send()
            .await
            .map_err(|e| format!("patch config error: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_proxy_groups(state: State<'_, AppState>) -> Result<Vec<ProxyGroup>, String> {
    if !state.mihomo.is_running() {
        return Ok(vec![]);
    }

    let port = state.config.lock().unwrap().controller_port;
    let secret = state.config.lock().unwrap().secret.clone();

    let client = reqwest::Client::new();
    let mut req = client.get(format!("http://127.0.0.1:{}/proxies", port));
    if !secret.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", secret));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("get proxies error: {}", e))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("parse proxies error: {}", e))?;

    let proxies = data["proxies"].as_object().ok_or("invalid proxies")?;
    let mut groups = Vec::new();

    for (name, proxy) in proxies {
        if let Some(all) = proxy["all"].as_array() {
            if !all.is_empty() {
                groups.push(ProxyGroup {
                    name: name.clone(),
                    proxy_type: proxy["type"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string(),
                    now: proxy["now"].as_str().unwrap_or("").to_string(),
                    all: all
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                });
            }
        }
    }

    Ok(groups)
}

#[tauri::command]
pub async fn select_proxy(
    group: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let port = state.config.lock().unwrap().controller_port;
    let secret = state.config.lock().unwrap().secret.clone();

    let client = reqwest::Client::new();
    let url = format!(
        "http://127.0.0.1:{}/proxies/{}",
        port,
        urlencoding::encode(&group)
    );
    let mut req = client.put(&url);
    if !secret.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", secret));
    }
    req.json(&serde_json::json!({"name": name}))
        .send()
        .await
        .map_err(|e| format!("select proxy error: {}", e))?;

    Ok(())
}
