use serde::{Deserialize, Serialize};
use crate::error::AgentError;
use reqwest::Client;

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
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
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
            // Store API key for future requests
            self.api_key = Some(registration_response.api_key.clone());
            Ok(registration_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("Registration failed: {}", error_text).into())
        }
    }

    pub fn get_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }
}