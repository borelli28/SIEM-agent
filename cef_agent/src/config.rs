use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub api_key: String,
    pub host_id: String,
    pub account_id: String,
    pub watch_paths: Vec<String>,
}

impl AgentConfig {
    pub fn load() -> Option<Self> {
        let config_path = "agent_config.json";
        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let config_path = "agent_config.json";
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)
    }
}