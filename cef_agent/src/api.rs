use serde::{Deserialize, Serialize};
use crate::config::AgentConfig;
use std::collections::HashMap;
use crate::error::AgentError;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use std::path::PathBuf;
use reqwest::Client;
use std::sync::Arc;

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

#[derive(Debug)]
struct FileUploadState {
    last_successful_upload: DateTime<Utc>,
    upload_failed: bool,
}

pub struct ApiClient {
    client: Client,
    api_key: Option<String>,
    config: Option<AgentConfig>,
    file_states: Arc<RwLock<HashMap<PathBuf, FileUploadState>>>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
            config: None,
            file_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(config.api_key.clone()),
            config: Some(config),
            file_states: Arc::new(RwLock::new(HashMap::new())),
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

        // Create file
        let file_bytes = tokio::fs::read(&path).await?;
        let file_part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(path.file_name().unwrap().to_string_lossy().to_string())
            .mime_str("application/octet-stream")?;

        // Create the multipart form
        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("account_id", config.account_id.clone())
            .text("host_id", config.host_id.clone())
            .text("api_key", config.api_key.clone());

        let response = self.client
            .post(&format!("{}/upload", API_BASE_URL))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AgentError::ApiError(Box::new(e)))?;

        // Upload unsuccessful
        if !response.status().is_success() {
            let mut states = self.file_states.write().await;
            let timestamp = states
                .get(&path)
                .map(|state| state.last_successful_upload)
                .unwrap_or_else(|| Utc::now());

            states.insert(path.clone(), FileUploadState {
                last_successful_upload: timestamp,
                upload_failed: true,
            });

            return Err(AgentError::ApiError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to upload log: {} - {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                )
            ))));
        }

        // Upload successful
        println!("Successfully uploaded log: {:?}", path);
        let mut states = self.file_states.write().await;
        states.insert(path.clone(), FileUploadState {
            last_successful_upload: Utc::now(),
            upload_failed: false,
        });

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

        // Heartbeat successful, retry failed uploads
        self.retry_uploads().await?;

        Ok(())
    }

    async fn retry_uploads(&self) -> Result<(), AgentError> {
        let states = self.file_states.read().await;
        let failed_uploads: Vec<PathBuf> = states
            .iter()
            .filter(|(_, state)| state.upload_failed)
            .map(|(path, _)| path.clone())
            .collect();

        for path in failed_uploads {
            if let Err(e) = self.upload_log(path.clone()).await {
                eprintln!("Failed to retry upload for {:?}: {}", path, e);
            }
        }

        Ok(())
    }
}