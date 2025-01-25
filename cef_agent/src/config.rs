use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub api_key: String,
    pub host_id: String,
    pub account_id: String,
    pub watch_paths: Vec<String>,
    pub siem_url: String
}

impl AgentConfig {
    pub fn load() -> Option<Self> {
        let exe_path = env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        let config_path = exe_dir.join("agent_config.json");

        if config_path.exists() {
            let content = fs::read_to_string(config_path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine executable directory"
            )
        })?;

        let config_path = exe_dir.join("agent_config.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)
    }
}