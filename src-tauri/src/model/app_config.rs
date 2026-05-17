use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::core::mihomo_manager::MihomoManager;
use super::subscription::Subscription;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mixed_port: u16,
    pub controller_port: u16,
    pub secret: String,
    pub mode: String,
    pub auto_start: bool,
    pub custom_rules: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mixed_port: 7890,
            controller_port: 9090,
            secret: uuid::Uuid::new_v4().to_string(),
            mode: "rule".to_string(),
            auto_start: false,
            custom_rules: vec![],
        }
    }
}

pub struct AppState {
    pub mihomo: MihomoManager,
    pub config: Mutex<AppConfig>,
    pub subscriptions: Mutex<Vec<Subscription>>,
}
