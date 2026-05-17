use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::core::system_proxy;
use crate::model::app_config::AppState;
use crate::utils::paths;

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItemBuilder::with_id("toggle", "Enable Proxy").build(app)?;
    let mode_global = MenuItemBuilder::with_id("mode_global", "Global Mode").build(app)?;
    let mode_rule = MenuItemBuilder::with_id("mode_rule", "Rule Mode").build(app)?;
    let mode_direct = MenuItemBuilder::with_id("mode_direct", "Direct Mode").build(app)?;
    let update_subs =
        MenuItemBuilder::with_id("update_subs", "Update Subscriptions").build(app)?;
    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle)
        .item(&sep1)
        .item(&mode_global)
        .item(&mode_rule)
        .item(&mode_direct)
        .item(&sep2)
        .item(&update_subs)
        .item(&sep3)
        .item(&show)
        .item(&sep4)
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "toggle" => {
                    let state = app.state::<AppState>();
                    if state.mihomo.is_running() {
                        system_proxy::disable_system_proxy().ok();
                        state.mihomo.stop().ok();
                    } else {
                        let mihomo_dir = paths::mihomo_dir(app);
                        let config = state.config.lock().unwrap().clone();
                        let subs = state.subscriptions.lock().unwrap().clone();
                        if let Ok(yaml) =
                            crate::core::config_generator::generate_config(&config, &subs)
                        {
                            std::fs::write(mihomo_dir.join("config.yaml"), yaml).ok();
                        }
                        state.mihomo.start(app, &mihomo_dir).ok();
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        system_proxy::enable_system_proxy(config.mixed_port).ok();
                    }
                }
                "mode_global" | "mode_rule" | "mode_direct" => {
                    let mode = match id {
                        "mode_global" => "global",
                        "mode_rule" => "rule",
                        "mode_direct" => "direct",
                        _ => return,
                    };
                    let state = app.state::<AppState>();
                    {
                        let mut config = state.config.lock().unwrap();
                        config.mode = mode.to_string();
                        paths::save_app_config(app, &config);
                    }
                    if state.mihomo.is_running() {
                        let config = state.config.lock().unwrap();
                        let port = config.controller_port;
                        let secret = config.secret.clone();
                        let mode = mode.to_string();
                        tauri::async_runtime::spawn(async move {
                            let client = reqwest::Client::new();
                            let mut req = client
                                .patch(format!("http://127.0.0.1:{}/configs", port));
                            if !secret.is_empty() {
                                req = req.header(
                                    "Authorization",
                                    format!("Bearer {}", secret),
                                );
                            }
                            req.json(&serde_json::json!({"mode": mode}))
                                .send()
                                .await
                                .ok();
                        });
                    }
                }
                "update_subs" => {
                    let state = app.state::<AppState>();
                    let app_clone = app.clone();
                    let subs = state.subscriptions.lock().unwrap().clone();
                    let mihomo_dir = paths::mihomo_dir(app);
                    tauri::async_runtime::spawn(async move {
                        let mut updated_subs = subs;
                        for sub in &mut updated_subs {
                            crate::core::subscription::update_subscription(sub, &mihomo_dir)
                                .await
                                .ok();
                        }
                        let state = app_clone.state::<AppState>();
                        let mut state_subs = state.subscriptions.lock().unwrap();
                        *state_subs = updated_subs;
                        paths::save_subscriptions(&app_clone, &state_subs);
                    });
                }
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
                "quit" => {
                    let state = app.state::<AppState>();
                    system_proxy::disable_system_proxy().ok();
                    state.mihomo.stop().ok();
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
