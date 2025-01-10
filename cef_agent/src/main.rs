mod api;
mod error;
mod config;

use std::sync::mpsc::RecvTimeoutError;
use notify::{Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::io::{self, Write};
use std::time::Duration;
use error::AgentError;
use std::path::Path;
use crate::api::{ApiClient, AgentRegistration, RegistrationResponse};
use crate::config::AgentConfig;

fn prompt(message: &str) -> Result<String, AgentError> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_watch_paths() -> Result<Vec<String>, AgentError> {
    let mut paths = Vec::new();
    loop {
        let path = prompt("Enter path to watch (or 'done' to finish): ")?;
        if path.to_lowercase() == "done" {
            break;
        }
        if Path::new(&path).exists() {
            paths.push(path);
        } else {
            println!("Path does not exist: {}", path);
        }
    }
    Ok(paths)
}

async fn register_agent(host_id: String, account_id: String, hostname: String) 
    -> Result<RegistrationResponse, AgentError> {
    let mut client = ApiClient::new();
    let registration = AgentRegistration {
        id: String::new(),
        api_key: String::new(),
        host_id,
        account_id,
        ip_address: Some("127.0.0.1".to_string()),
        hostname: Some(hostname),
        status: "Active".to_string()
    };

    let response = client.register(registration).await?;
    println!("Registration successful! Agent ID: {}", response.agent_id);
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    let config = match AgentConfig::load() {
        Some(config) => config,
        None => {
            println!("No configuration found. Let's set up the agent.");

            let host_id = prompt("Enter host ID: ")?;
            let account_id = prompt("Enter account ID: ")?;
            let hostname = prompt("Enter hostname: ")?;
            let siem_url = prompt("Enter SIEM URL (default: http://localhost:4200): ")?;
            let siem_url = if siem_url.is_empty() {
                "http://localhost:4200".to_string()
            } else {
                siem_url
            };

            println!("Now let's set up the paths to monitor.");
            let watch_paths = prompt_watch_paths()?;

            let response = register_agent(
                host_id.clone(),
                account_id.clone(),
                hostname
            ).await?;

            let config = AgentConfig {
                agent_id: response.agent_id,
                api_key: response.api_key,
                host_id,
                account_id,
                watch_paths,
                siem_url,
            };
            config.save()?;
            config
        }
    };

    // Verify paths exist
    for path in &config.watch_paths {
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
    for path in &config.watch_paths {
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