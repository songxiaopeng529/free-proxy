use std::process::Command;

use crate::model::proxy::SystemProxyStatus;
use crate::utils::network::get_active_network_service;

pub fn enable_system_proxy(port: u16) -> Result<(), String> {
    let service = get_active_network_service()?;
    let port_str = port.to_string();

    let commands = [
        vec!["-setwebproxy", &service, "127.0.0.1", &port_str],
        vec!["-setwebproxystate", &service, "on"],
        vec!["-setsecurewebproxy", &service, "127.0.0.1", &port_str],
        vec!["-setsecurewebproxystate", &service, "on"],
        vec!["-setsocksfirewallproxy", &service, "127.0.0.1", &port_str],
        vec!["-setsocksfirewallproxystate", &service, "on"],
        vec![
            "-setproxybypassdomains",
            &service,
            "127.0.0.1",
            "localhost",
            "*.local",
            "192.168.*",
            "10.*",
            "172.16.*",
        ],
    ];

    for args in &commands {
        let output = Command::new("networksetup")
            .args(args)
            .output()
            .map_err(|e| format!("networksetup error: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("networksetup {:?} failed: {}", args, stderr);
        }
    }

    Ok(())
}

pub fn disable_system_proxy() -> Result<(), String> {
    let service = get_active_network_service()?;

    let commands = [
        vec!["-setwebproxystate", &*service, "off"],
        vec!["-setsecurewebproxystate", &*service, "off"],
        vec!["-setsocksfirewallproxystate", &*service, "off"],
    ];

    for args in &commands {
        let output = Command::new("networksetup")
            .args(args)
            .output()
            .map_err(|e| format!("networksetup error: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("networksetup {:?} failed: {}", args, stderr);
        }
    }

    Ok(())
}

pub fn get_system_proxy_status() -> Result<SystemProxyStatus, String> {
    let service = get_active_network_service()?;

    let http = get_proxy_state(&service, "-getwebproxy");
    let https = get_proxy_state(&service, "-getsecurewebproxy");
    let socks = get_proxy_state(&service, "-getsocksfirewallproxy");

    let enabled = http.is_some() || https.is_some() || socks.is_some();

    Ok(SystemProxyStatus {
        enabled,
        http_proxy: http,
        https_proxy: https,
        socks_proxy: socks,
    })
}

fn get_proxy_state(service: &str, flag: &str) -> Option<String> {
    let output = Command::new("networksetup")
        .args([flag, service])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut enabled = false;
    let mut server = String::new();
    let mut port = String::new();

    for line in stdout.lines() {
        if line.starts_with("Enabled:") {
            enabled = line.contains("Yes");
        } else if line.starts_with("Server:") {
            server = line.trim_start_matches("Server:").trim().to_string();
        } else if line.starts_with("Port:") {
            port = line.trim_start_matches("Port:").trim().to_string();
        }
    }

    if enabled && !server.is_empty() {
        Some(format!("{}:{}", server, port))
    } else {
        None
    }
}
