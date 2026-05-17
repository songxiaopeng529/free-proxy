use std::sync::Mutex;
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

use crate::model::proxy::MihomoStatus;

pub struct MihomoManager {
    child: Mutex<Option<CommandChild>>,
    pub controller_port: u16,
    pub mixed_port: u16,
    pub secret: String,
    running: Mutex<bool>,
}

impl MihomoManager {
    pub fn new(controller_port: u16, mixed_port: u16, secret: &str) -> Self {
        Self {
            child: Mutex::new(None),
            controller_port,
            mixed_port,
            secret: secret.to_string(),
            running: Mutex::new(false),
        }
    }

    pub fn start(&self, app: &tauri::AppHandle, mihomo_dir: &std::path::Path) -> Result<(), String> {
        if *self.running.lock().unwrap() {
            return Ok(());
        }

        let dir_str = mihomo_dir
            .to_str()
            .ok_or("invalid mihomo dir path")?
            .to_string();

        let (mut rx, child) = app
            .shell()
            .sidecar("mihomo")
            .map_err(|e| format!("failed to create sidecar command: {}", e))?
            .args(["-d", &dir_str])
            .spawn()
            .map_err(|e| format!("failed to spawn mihomo: {}", e))?;

        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        log::info!("[mihomo] {}", String::from_utf8_lossy(&line));
                    }
                    CommandEvent::Stderr(line) => {
                        log::warn!("[mihomo] {}", String::from_utf8_lossy(&line));
                    }
                    CommandEvent::Terminated(payload) => {
                        log::info!("[mihomo] terminated: {:?}", payload);
                        app_handle.emit("mihomo://status-change", false).ok();
                        break;
                    }
                    _ => {}
                }
            }
        });

        *self.child.lock().unwrap() = Some(child);
        *self.running.lock().unwrap() = true;

        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut child_lock = self.child.lock().unwrap();
        if let Some(child) = child_lock.take() {
            child.kill().map_err(|e| format!("failed to kill mihomo: {}", e))?;
        }
        *self.running.lock().unwrap() = false;
        Ok(())
    }

    pub fn restart(&self, app: &tauri::AppHandle, mihomo_dir: &std::path::Path) -> Result<(), String> {
        self.stop()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        self.start(app, mihomo_dir)
    }

    pub async fn health_check(&self) -> bool {
        let url = format!("http://127.0.0.1:{}/version", self.controller_port);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap();

        let mut req = client.get(&url);
        if !self.secret.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.secret));
        }

        req.send().await.is_ok()
    }

    pub async fn status(&self) -> MihomoStatus {
        let running = *self.running.lock().unwrap();
        if !running {
            return MihomoStatus {
                running: false,
                version: None,
            };
        }

        let url = format!("http://127.0.0.1:{}/version", self.controller_port);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap();

        let mut req = client.get(&url);
        if !self.secret.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.secret));
        }

        match req.send().await {
            Ok(resp) => {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let version = body["version"].as_str().map(|s| s.to_string());
                    MihomoStatus {
                        running: true,
                        version,
                    }
                } else {
                    MihomoStatus {
                        running: true,
                        version: None,
                    }
                }
            }
            Err(_) => MihomoStatus {
                running,
                version: None,
            },
        }
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}
