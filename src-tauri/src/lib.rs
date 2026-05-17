mod cmd;
mod core;
mod model;
mod tray;
mod utils;

use model::app_config::AppState;
use core::mihomo_manager::MihomoManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Ensure directories exist
            utils::paths::mihomo_dir(app.handle());

            // Load config
            let config = utils::paths::load_app_config(app.handle());
            let subscriptions = utils::paths::load_subscriptions(app.handle());

            // Initialize state
            let mihomo = MihomoManager::new(
                config.controller_port,
                config.mixed_port,
                &config.secret,
            );

            app.manage(AppState {
                mihomo,
                config: std::sync::Mutex::new(config),
                subscriptions: std::sync::Mutex::new(subscriptions),
            });

            // Create tray
            tray::tray::create_tray(app.handle())?;

            // Clean up stale system proxy on startup
            cleanup_stale_proxy(app.handle());

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().ok();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            cmd::proxy::start_proxy,
            cmd::proxy::stop_proxy,
            cmd::proxy::restart_proxy,
            cmd::proxy::get_proxy_status,
            cmd::subscription::add_subscription,
            cmd::subscription::remove_subscription,
            cmd::subscription::update_subscription,
            cmd::subscription::update_all_subscriptions,
            cmd::subscription::list_subscriptions,
            cmd::config::get_app_config,
            cmd::config::set_proxy_mode,
            cmd::config::get_proxy_groups,
            cmd::config::select_proxy,
            cmd::system_proxy::enable_system_proxy,
            cmd::system_proxy::disable_system_proxy,
            cmd::system_proxy::get_system_proxy_status,
            cmd::rules::get_rules,
            cmd::rules::set_custom_rules,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn cleanup_stale_proxy(app: &tauri::AppHandle) {
    if let Ok(status) = core::system_proxy::get_system_proxy_status() {
        if status.enabled {
            let config = utils::paths::load_app_config(app);
            let our_proxy = format!("127.0.0.1:{}", config.mixed_port);
            let is_ours = status
                .http_proxy
                .as_ref()
                .map(|p| p == &our_proxy)
                .unwrap_or(false);

            if is_ours {
                log::info!("cleaning up stale system proxy from previous session");
                core::system_proxy::disable_system_proxy().ok();
            }
        }
    }
}
