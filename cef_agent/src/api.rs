use serde::{Deserialize, Serialize};
use crate::error::AgentError;
use crate::config::AgentConfig;
use reqwest::Client;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use chrono;

const API_BASE_URL: &str = "http://localhost:4200/backend/agent";

#[derive(Debug, Serialize)]
pub struct AgentRegistration {
    pub id: String,
    pub api_key: String,
    pub host_id: String,
    pub account_id: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
    pub status: String
}

#[derive(Deserialize)]
pub struct RegistrationResponse {
    pub status: String,
    pub agent_id: String,
    pub api_key: String,
}

pub struct ApiClient {
    client: Client,
    api_key: Option<String>,
    config: Option<AgentConfig>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
            config: None,
        }
    }

    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(config.api_key.clone()),
            config: Some(config),
        }
    }

    pub async fn register(&mut self, registration: AgentRegistration) -> Result<RegistrationResponse, AgentError> {
        let response = self.client
            .post(&format!("{}/register", API_BASE_URL))
            .json(&registration)
            .send()
            .await?;

        if response.status().is_success() {
            let registration_response = response.json::<RegistrationResponse>().await?;
            self.api_key = Some(registration_response.api_key.clone());
            Ok(registration_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("Registration failed: {}", error_text).into())
        }
    }

    pub async fn upload_log(&self, path: PathBuf) -> Result<(), AgentError> {
        let config = self.config.as_ref()
            .ok_or(AgentError::ValidationError("No configuration available".to_string()))?;

        // Read the file
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Prepare the upload payload
        let payload = serde_json::json!({
            "agent_id": config.agent_id,
            "host_id": config.host_id,
            "account_id": config.account_id,
            "file_path": path.to_string_lossy().to_string(),
            "content": contents,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Send to SIEM API
        let response = self.client
            .post(&format!("{}/upload", API_BASE_URL))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| AgentError::ApiError(Box::new(e)))?;

        if !response.status().is_success() {
            return Err(AgentError::ApiError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to upload log: {} - {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                )
            ))));
        }

        println!("Successfully uploaded log: {:?}", path);
        Ok(())
    }

    pub async fn send_heartbeat(&self) -> Result<(), AgentError> {
        let config = self.config.as_ref()
            .ok_or(AgentError::ValidationError("No configuration available".to_string()))?;

        let payload = serde_json::json!({
            "agent_id": config.agent_id,
            "host_id": config.host_id,
            "account_id": config.account_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let response = self.client
            .post(&format!("{}/heartbeat", API_BASE_URL))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| AgentError::ApiError(Box::new(e)))?;

        if !response.status().is_success() {
            return Err(AgentError::ApiError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to send heartbeat: {} - {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                )
            ))));
        }

        Ok(())
    }

    pub fn get_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }
}