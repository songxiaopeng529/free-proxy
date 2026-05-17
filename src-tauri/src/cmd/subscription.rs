use tauri::State;

use crate::core::config_generator;
use crate::core::subscription as sub_core;
use crate::model::app_config::AppState;
use crate::model::subscription::Subscription;
use crate::utils::paths;

#[tauri::command]
pub async fn add_subscription(
    url: String,
    name: Option<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Subscription, String> {
    let mihomo_dir = paths::mihomo_dir(&app);
    let sub = sub_core::add_subscription(&url, name.as_deref(), &mihomo_dir).await?;

    {
        let mut subs = state.subscriptions.lock().unwrap();
        subs.push(sub.clone());
        paths::save_subscriptions(&app, &subs);
    }

    regenerate_and_reload(&app, &state)?;
    Ok(sub)
}

#[tauri::command]
pub fn remove_subscription(
    id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mihomo_dir = paths::mihomo_dir(&app);

    {
        let mut subs = state.subscriptions.lock().unwrap();
        subs.retain(|s| s.id != id);
        paths::save_subscriptions(&app, &subs);
    }

    // Remove provider file
    let provider_path = mihomo_dir.join("providers").join(format!("{}.yaml", id));
    std::fs::remove_file(provider_path).ok();

    regenerate_and_reload(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub async fn update_subscription(
    id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Subscription, String> {
    let mihomo_dir = paths::mihomo_dir(&app);

    let mut sub = {
        let subs = state.subscriptions.lock().unwrap();
        subs.iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or("subscription not found")?
    };

    sub_core::update_subscription(&mut sub, &mihomo_dir).await?;

    {
        let mut subs = state.subscriptions.lock().unwrap();
        if let Some(s) = subs.iter_mut().find(|s| s.id == id) {
            *s = sub.clone();
        }
        paths::save_subscriptions(&app, &subs);
    }

    regenerate_and_reload(&app, &state)?;
    Ok(sub)
}

#[tauri::command]
pub async fn update_all_subscriptions(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<Subscription>, String> {
    let mihomo_dir = paths::mihomo_dir(&app);

    let mut subs = state.subscriptions.lock().unwrap().clone();

    for sub in &mut subs {
        if let Err(e) = sub_core::update_subscription(sub, &mihomo_dir).await {
            log::warn!("failed to update subscription {}: {}", sub.name, e);
        }
    }

    {
        let mut state_subs = state.subscriptions.lock().unwrap();
        *state_subs = subs.clone();
        paths::save_subscriptions(&app, &state_subs);
    }

    regenerate_and_reload(&app, &state)?;
    Ok(subs)
}

#[tauri::command]
pub fn list_subscriptions(state: State<'_, AppState>) -> Result<Vec<Subscription>, String> {
    Ok(state.subscriptions.lock().unwrap().clone())
}

fn regenerate_and_reload(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    if !state.mihomo.is_running() {
        return Ok(());
    }

    let mihomo_dir = paths::mihomo_dir(app);
    let config = state.config.lock().unwrap().clone();
    let subs = state.subscriptions.lock().unwrap().clone();

    let yaml = config_generator::generate_config(&config, &subs)?;
    std::fs::write(mihomo_dir.join("config.yaml"), &yaml)
        .map_err(|e| format!("write config error: {}", e))?;

    // Reload via API
    let port = config.controller_port;
    let secret = config.secret.clone();
    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::new();
        let mut req = client.put(format!("http://127.0.0.1:{}/configs?force=true", port));
        if !secret.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", secret));
        }
        req.json(&serde_json::json!({"path": ""}))
            .send()
            .await
            .ok();
    });

    Ok(())
}
