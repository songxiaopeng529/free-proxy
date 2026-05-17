use std::fs;
use std::path::PathBuf;
use tauri::Manager;

use crate::model::app_config::AppConfig;
use crate::model::subscription::Subscription;

pub fn app_data_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn mihomo_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app_data_dir(app).join("mihomo");
    fs::create_dir_all(&dir).ok();
    fs::create_dir_all(dir.join("providers")).ok();
    dir
}

pub fn config_path(app: &tauri::AppHandle) -> PathBuf {
    app_data_dir(app).join("app_config.json")
}

pub fn subscriptions_path(app: &tauri::AppHandle) -> PathBuf {
    app_data_dir(app).join("subscriptions.json")
}

pub fn load_app_config(app: &tauri::AppHandle) -> AppConfig {
    let path = config_path(app);
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        let config = AppConfig::default();
        save_app_config(app, &config);
        config
    }
}

pub fn save_app_config(app: &tauri::AppHandle, config: &AppConfig) {
    let path = config_path(app);
    let data = serde_json::to_string_pretty(config).unwrap();
    fs::write(path, data).ok();
}

pub fn load_subscriptions(app: &tauri::AppHandle) -> Vec<Subscription> {
    let path = subscriptions_path(app);
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

pub fn save_subscriptions(app: &tauri::AppHandle, subs: &[Subscription]) {
    let path = subscriptions_path(app);
    let data = serde_json::to_string_pretty(subs).unwrap();
    fs::write(path, data).ok();
}
