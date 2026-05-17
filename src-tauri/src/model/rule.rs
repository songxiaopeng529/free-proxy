pub const DEFAULT_RULES: &[&str] = &[
    "GEOIP,CN,DIRECT",
    "GEOSITE,cn,DIRECT",
    "GEOSITE,geolocation-!cn,PROXY",
    "MATCH,PROXY",
];
