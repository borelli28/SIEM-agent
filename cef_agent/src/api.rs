use serde::{Deserialize, Serialize};
use crate::error::AgentError;
use crate::config::AgentConfig;
use reqwest::Client;
use std::path::PathBuf;
use reqwest::multipart::{Form, Part};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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

    pub fn get_config(&self) -> &AgentConfig {
        self.config.as_ref().unwrap()
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

        // Read file
        let mut file = File::open(&path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Create file part
        let file_part = Part::bytes(buffer)
            .file_name(path.file_name().unwrap().to_string_lossy().to_string())
            .mime_str("application/octet-stream")?;

        // Create the form
        let form = Form::new()
            .part("log_file", file_part)
            .text("api_key", config.api_key.clone())
            .text("account_id", config.account_id.clone())
            .text("host_id", config.host_id.clone());

        // Send to SIEM API
        let response = self.client
            .post(&format!("{}/upload", API_BASE_URL))
            .multipart(form)
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
            "api_key": config.api_key,
        });

        let response = self.client
            .post(&format!("{}/heartbeat", API_BASE_URL))
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