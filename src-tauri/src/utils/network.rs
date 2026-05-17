use std::process::Command;

pub fn get_active_network_service() -> Result<String, String> {
    let output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()
        .map_err(|e| format!("failed to list network services: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines().skip(1) {
        let service = line.trim_start_matches('*').trim();
        if service.is_empty() {
            continue;
        }

        let info = Command::new("networksetup")
            .args(["-getinfo", service])
            .output();

        if let Ok(info_output) = info {
            let info_str = String::from_utf8_lossy(&info_output.stdout);
            for info_line in info_str.lines() {
                if info_line.starts_with("IP address:") {
                    let ip = info_line.trim_start_matches("IP address:").trim();
                    if !ip.is_empty() && ip != "none" {
                        return Ok(service.to_string());
                    }
                }
            }
        }
    }

    Ok("Wi-Fi".to_string())
}
