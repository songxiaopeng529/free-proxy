use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use crate::model::proxy::MihomoStatus;

pub struct MihomoManager {
    child: Mutex<Option<Child>>,
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

        let binary_path = resolve_mihomo_path(app)?;
        eprintln!("[mihomo] binary: {}", binary_path);
        eprintln!("[mihomo] data dir: {}", dir_str);

        let child = Command::new(&binary_path)
            .args(["-d", &dir_str])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn mihomo at {}: {}", binary_path, e))?;

        let pid = child.id();
        eprintln!("[mihomo] spawned with pid {}", pid);

        *self.child.lock().unwrap() = Some(child);
        *self.running.lock().unwrap() = true;

        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut child_lock = self.child.lock().unwrap();
        if let Some(mut child) = child_lock.take() {
            child.kill().ok();
            child.wait().ok();
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
            Err(_) => {
                // mihomo is not responding, mark as not running
                *self.running.lock().unwrap() = false;
                MihomoStatus {
                    running: false,
                    version: None,
                }
            }
        }
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

fn resolve_mihomo_path(app: &tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;

    let mut searched = Vec::new();

    let resource_dir = app.path().resource_dir()
        .map_err(|e| format!("failed to get resource dir: {}", e))?;

    let bundle_path = resource_dir.join("binaries").join("mihomo");
    searched.push(bundle_path.display().to_string());
    if bundle_path.exists() {
        return Ok(bundle_path.to_string_lossy().to_string());
    }

    #[cfg(target_arch = "aarch64")]
    let triple_name = "mihomo-aarch64-apple-darwin";
    #[cfg(target_arch = "x86_64")]
    let triple_name = "mihomo-x86_64-apple-darwin";

    let triple_path = resource_dir.join("binaries").join(triple_name);
    searched.push(triple_path.display().to_string());
    if triple_path.exists() {
        return Ok(triple_path.to_string_lossy().to_string());
    }

    let dev_dir = std::env::current_dir().unwrap_or_default();
    let exe_dir = std::env::current_exe()
        .map(|p| p.parent().unwrap_or(p.as_path()).to_path_buf())
        .unwrap_or_default();

    for base in [&dev_dir, &exe_dir] {
        let dev_path = base.join("binaries").join(triple_name);
        searched.push(dev_path.display().to_string());
        if dev_path.exists() {
            return Ok(dev_path.to_string_lossy().to_string());
        }
    }

    let src_tauri = dev_dir.join("src-tauri").join("binaries");
    let fallback = src_tauri.join(triple_name);
    searched.push(fallback.display().to_string());
    if fallback.exists() {
        return Ok(fallback.to_string_lossy().to_string());
    }

    Err(format!(
        "mihomo binary not found. Searched paths:\n{}",
        searched.join("\n")
    ))
}
