use crate::api::{ApiClient, AgentRegistration, RegistrationResponse};
use crate::config::AgentConfig;
use crate::error::AgentError;

pub async fn register_agent(host_id: String, account_id: String, hostname: String) 
    -> Result<RegistrationResponse, AgentError> {
    let mut client = ApiClient::new();
    let registration = AgentRegistration {
        id: String::new(),
        api_key: String::new(),
        host_id: host_id.clone(),
        account_id: account_id.clone(),
        ip_address: Some("127.0.0.1".to_string()),
        hostname: Some(hostname),
        status: "Active".to_string()
    };

    let response = client.register(registration).await?;
    println!("Registration successful!");

    let config = AgentConfig {
        agent_id: response.agent_id.clone(),
        api_key: response.api_key.clone(),
        host_id: host_id.clone(),
        account_id: account_id.clone(),
        watch_paths: Vec::new(),
        siem_url: "http://localhost:4200".to_string(),
    };
    config.save()?;

    Ok(response)
}