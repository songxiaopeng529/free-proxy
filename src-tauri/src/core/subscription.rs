use std::fs;
use std::path::Path;

use crate::model::subscription::{Subscription, TrafficInfo};

pub async fn fetch_subscription(url: &str) -> Result<(Vec<serde_yaml::Value>, Option<TrafficInfo>), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("http client error: {}", e))?;

    let resp = client
        .get(url)
        .header("User-Agent", "clash.meta")
        .send()
        .await
        .map_err(|e| format!("fetch error: {}", e))?;

    let traffic_info = parse_traffic_header(
        resp.headers()
            .get("subscription-userinfo")
            .and_then(|v| v.to_str().ok()),
    );

    let body = resp
        .text()
        .await
        .map_err(|e| format!("read body error: {}", e))?;

    let proxies = parse_clash_yaml(&body)?;
    Ok((proxies, traffic_info))
}

fn parse_clash_yaml(content: &str) -> Result<Vec<serde_yaml::Value>, String> {
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(content).map_err(|e| format!("yaml parse error: {}", e))?;

    let proxies = yaml
        .get("proxies")
        .and_then(|p| p.as_sequence())
        .ok_or("no proxies found in subscription")?;

    Ok(proxies.clone())
}

fn parse_traffic_header(header: Option<&str>) -> Option<TrafficInfo> {
    let header = header?;
    let mut upload = 0u64;
    let mut download = 0u64;
    let mut total = 0u64;
    let mut expire = None;

    for part in header.split(';') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "upload" => upload = value.parse().unwrap_or(0),
                "download" => download = value.parse().unwrap_or(0),
                "total" => total = value.parse().unwrap_or(0),
                "expire" => expire = value.parse().ok(),
                _ => {}
            }
        }
    }

    Some(TrafficInfo {
        upload,
        download,
        total,
        expire,
    })
}

pub fn save_provider_file(
    proxies: &[serde_yaml::Value],
    path: &Path,
) -> Result<(), String> {
    let mut map = serde_yaml::Mapping::new();
    map.insert(
        serde_yaml::Value::String("proxies".to_string()),
        serde_yaml::Value::Sequence(proxies.to_vec()),
    );
    let yaml =
        serde_yaml::to_string(&serde_yaml::Value::Mapping(map)).map_err(|e| e.to_string())?;
    fs::write(path, yaml).map_err(|e| format!("write provider file error: {}", e))
}

pub async fn add_subscription(
    url: &str,
    name: Option<&str>,
    mihomo_dir: &Path,
) -> Result<Subscription, String> {
    let (proxies, traffic_info) = fetch_subscription(url).await?;
    let id = uuid::Uuid::new_v4().to_string();

    let provider_path = mihomo_dir.join("providers").join(format!("{}.yaml", id));
    save_provider_file(&proxies, &provider_path)?;

    let sub_name = name
        .map(|n| n.to_string())
        .unwrap_or_else(|| {
            extract_name_from_url(url).unwrap_or_else(|| format!("Sub {}", &id[..8]))
        });

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Ok(Subscription {
        id,
        name: sub_name,
        url: url.to_string(),
        last_updated: Some(now),
        node_count: proxies.len(),
        traffic_info,
    })
}

pub async fn update_subscription(
    sub: &mut Subscription,
    mihomo_dir: &Path,
) -> Result<(), String> {
    let (proxies, traffic_info) = fetch_subscription(&sub.url).await?;

    let provider_path = mihomo_dir.join("providers").join(format!("{}.yaml", sub.id));
    save_provider_file(&proxies, &provider_path)?;

    sub.node_count = proxies.len();
    sub.traffic_info = traffic_info;
    sub.last_updated = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    );

    Ok(())
}

fn extract_name_from_url(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed.host_str().map(|h| h.to_string())
}
