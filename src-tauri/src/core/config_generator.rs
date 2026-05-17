use serde_yaml::Value;

use crate::model::app_config::AppConfig;
use crate::model::rule::DEFAULT_RULES;
use crate::model::subscription::Subscription;

pub fn generate_config(
    config: &AppConfig,
    subscriptions: &[Subscription],
) -> Result<String, String> {
    let mut map = serde_yaml::Mapping::new();

    map.insert(val("mixed-port"), Value::Number(config.mixed_port.into()));
    map.insert(val("allow-lan"), Value::Bool(false));
    map.insert(val("mode"), val_s(&config.mode));
    map.insert(val("log-level"), val_s("info"));
    map.insert(val("ipv6"), Value::Bool(false));
    map.insert(
        val("external-controller"),
        val_s(&format!("127.0.0.1:{}", config.controller_port)),
    );
    map.insert(val("secret"), val_s(&config.secret));

    // DNS
    let mut dns = serde_yaml::Mapping::new();
    dns.insert(val("enable"), Value::Bool(true));
    dns.insert(val("enhanced-mode"), val_s("fake-ip"));
    dns.insert(val("fake-ip-range"), val_s("198.18.0.1/16"));
    dns.insert(
        val("nameserver"),
        Value::Sequence(vec![
            val_s("https://dns.alidns.com/dns-query"),
            val_s("https://doh.pub/dns-query"),
        ]),
    );
    dns.insert(
        val("fallback"),
        Value::Sequence(vec![
            val_s("https://dns.google/dns-query"),
            val_s("https://cloudflare-dns.com/dns-query"),
        ]),
    );

    let mut fallback_filter = serde_yaml::Mapping::new();
    fallback_filter.insert(val("geoip"), Value::Bool(true));
    fallback_filter.insert(val("geoip-code"), val_s("CN"));
    dns.insert(
        val("fallback-filter"),
        Value::Mapping(fallback_filter),
    );
    map.insert(val("dns"), Value::Mapping(dns));

    // proxy-providers
    if !subscriptions.is_empty() {
        let mut providers = serde_yaml::Mapping::new();
        for sub in subscriptions {
            let mut provider = serde_yaml::Mapping::new();
            provider.insert(val("type"), val_s("file"));
            provider.insert(
                val("path"),
                val_s(&format!("./providers/{}.yaml", sub.id)),
            );
            let mut hc = serde_yaml::Mapping::new();
            hc.insert(val("enable"), Value::Bool(true));
            hc.insert(
                val("url"),
                val_s("https://cp.cloudflare.com/generate_204"),
            );
            hc.insert(val("interval"), Value::Number(300.into()));
            provider.insert(val("health-check"), Value::Mapping(hc));
            providers.insert(val_s(&sub.id), Value::Mapping(provider));
        }
        map.insert(val("proxy-providers"), Value::Mapping(providers));
    }

    // proxy-groups
    let mut groups = Vec::new();

    let mut select_group = serde_yaml::Mapping::new();
    select_group.insert(val("name"), val_s("PROXY"));
    select_group.insert(val("type"), val_s("select"));
    if !subscriptions.is_empty() {
        select_group.insert(val("include-all-providers"), Value::Bool(true));
    }
    select_group.insert(
        val("proxies"),
        Value::Sequence(vec![val_s("auto"), val_s("DIRECT")]),
    );
    groups.push(Value::Mapping(select_group));

    let mut auto_group = serde_yaml::Mapping::new();
    auto_group.insert(val("name"), val_s("auto"));
    auto_group.insert(val("type"), val_s("url-test"));
    if !subscriptions.is_empty() {
        auto_group.insert(val("include-all-providers"), Value::Bool(true));
    }
    auto_group.insert(
        val("url"),
        val_s("https://cp.cloudflare.com/generate_204"),
    );
    auto_group.insert(val("interval"), Value::Number(300.into()));
    auto_group.insert(val("tolerance"), Value::Number(150.into()));
    groups.push(Value::Mapping(auto_group));

    map.insert(val("proxy-groups"), Value::Sequence(groups));

    // rules
    let rules: Vec<Value> = if config.custom_rules.is_empty() {
        DEFAULT_RULES.iter().map(|r| val_s(r)).collect()
    } else {
        config.custom_rules.iter().map(|r| val_s(r)).collect()
    };
    map.insert(val("rules"), Value::Sequence(rules));

    // profile
    let mut profile = serde_yaml::Mapping::new();
    profile.insert(val("store-selected"), Value::Bool(true));
    profile.insert(val("store-fake-ip"), Value::Bool(true));
    map.insert(val("profile"), Value::Mapping(profile));

    serde_yaml::to_string(&Value::Mapping(map)).map_err(|e| format!("yaml error: {}", e))
}

fn val(s: &str) -> Value {
    Value::String(s.to_string())
}

fn val_s(s: &str) -> Value {
    Value::String(s.to_string())
}
