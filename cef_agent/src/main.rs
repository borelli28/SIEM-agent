mod api;
mod error;
mod config;

use std::sync::mpsc::RecvTimeoutError;
use notify::{Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::time::Duration;
use error::AgentError;
use std::path::Path;
use crate::api::{ApiClient, AgentRegistration, RegistrationResponse};
use crate::config::AgentConfig;

async fn register_agent(host_id: String, account_id: String, hostname: String) 
    -> Result<RegistrationResponse, AgentError> {
    let mut client = ApiClient::new();
    let registration = AgentRegistration {
        host_id,
        account_id,
        ip_address: Some("127.0.0.1:3001".to_string()),
        hostname: Some(hostname)
    };

    let response = client.register(registration).await?;
    println!("Registration successful! Agent ID: {}", response.agent_id);
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    let paths = vec![
        "/path/to/logs/app1.cef.log",
        "/path/to/logs/app2.cef.log"
    ];

    register_agent(
        "test-host".to_string(),
        "test-account".to_string(),
        "test.local".to_string()
    ).await?;

    let config = match AgentConfig::load() {
        Some(config) => config,
        None => {
            // Only register if no config exists
            let response = register_agent(
                "test-host".to_string(),
                "test-account".to_string(),
                "test.local".to_string()
            ).await?;

            let config = AgentConfig {
                agent_id: response.agent_id,
                api_key: response.api_key,
                host_id: "test-host".to_string(),
                account_id: "test-account".to_string(),
            };
            config.save()?;
            config
        }
    };

    // Verify paths exist
    for path in &paths {
        if !Path::new(path).exists() {
            eprintln!("Path does not exist: {}", path);
            continue;
        }
    }

    // Create a channel to receive events
    let (tx, rx) = channel();

    // Create watcher
    let mut watcher = notify::recommended_watcher(move |res| {
        match res {
            Ok(event) => tx.send(event).unwrap(),
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }).map_err(AgentError::NotifyError)?;

    // Add paths to watch list
    for path in &paths {
        watcher.watch(Path::new(path), RecursiveMode::NonRecursive)
            .map_err(AgentError::NotifyError)?;
    }

    // Event handling loop
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                // Handle file events
                if event.kind.is_modify() || event.kind.is_create() {
                    // Get the file path that triggered the event
                    for path in event.paths {
                        if path.extension().unwrap_or_default() == "log" {
                            println!("Change detected in: {:?}", path);
                            // TODO: Send to SIEM API
                        }
                    }
                }
            }
            Err(e) => {
                match e {
                    RecvTimeoutError::Timeout => continue,
                    RecvTimeoutError::Disconnected => {
                        eprintln!("Watch error: channel disconnected");
                        break Ok(());
                    }
                }
            }
        }
    }
}