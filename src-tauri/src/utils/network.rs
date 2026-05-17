use std::process::Command;

pub fn get_active_network_service() -> Result<String, String> {
    if let Some(service) = service_for_default_route()? {
        return Ok(service);
    }

    if service_has_ip("Wi-Fi") {
        return Ok("Wi-Fi".to_string());
    }

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

fn service_for_default_route() -> Result<Option<String>, String> {
    let output = Command::new("route")
        .args(["-n", "get", "default"])
        .output()
        .map_err(|e| format!("failed to get default route: {}", e))?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let interface = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("interface:"))
        .map(str::trim)
        .filter(|iface| !iface.is_empty());

    let Some(interface) = interface else {
        return Ok(None);
    };

    // VPN interfaces such as utun do not have a networksetup service. In that
    // case the system proxy still needs to be set on the underlying service.
    if interface.starts_with("utun") {
        return Ok(None);
    }

    service_for_device(interface)
}

fn service_for_device(device: &str) -> Result<Option<String>, String> {
    let output = Command::new("networksetup")
        .arg("-listnetworkserviceorder")
        .output()
        .map_err(|e| format!("failed to list network service order: {}", e))?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_service: Option<String> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with('(') && !line.contains("Hardware Port:") {
            if let Some((_, name)) = line.split_once(')') {
                current_service = Some(name.trim_start_matches('*').trim().to_string());
            }
            continue;
        }

        if line.contains(&format!("Device: {}", device)) {
            return Ok(current_service);
        }
    }

    Ok(None)
}

fn service_has_ip(service: &str) -> bool {
    let Ok(output) = Command::new("networksetup").args(["-getinfo", service]).output() else {
        return false;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| {
        line.starts_with("IP address:")
            && line
                .trim_start_matches("IP address:")
                .trim()
                .parse::<std::net::IpAddr>()
                .is_ok()
    })
}
