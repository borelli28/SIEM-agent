mod api;
mod error;
mod config;

use clap::{Command, Arg, ArgAction};
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

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    // Command line interface setup
    let matches = Command::new("CEF Agent")
        .version("1.0")
        .subcommand(Command::new("config")
            .about("Configure the agent")
            .subcommand(Command::new("add-path")
                .about("Add a path to monitor")
                .arg(Arg::new("path")
                    .required(true)
                    .help("Path to monitor")))
            .subcommand(Command::new("remove-path")
                .about("Remove a monitored path")
                .arg(Arg::new("path")
                    .required(true)
                    .help("Path to remove")))
            .subcommand(Command::new("list-paths")
                .about("List all monitored paths"))
            .subcommand(Command::new("set-url")
                .about("Set SIEM URL")
                .arg(Arg::new("url")
                    .required(true)
                    .help("SIEM URL"))))
        .get_matches();

    // Handle configuration commands if present
    if let Some(config_matches) = matches.subcommand_matches("config") {
        let mut config = AgentConfig::load().ok_or(AgentError::ValidationError("No configuration found".to_string()))?;

        match config_matches.subcommand() {
            Some(("add-path", sub_m)) => {
                let path = sub_m.get_one::<String>("path").unwrap();
                if Path::new(path).exists() {
                    config.watch_paths.push(path.to_string());
                    config.save()?;
                    println!("Added path: {}", path);
                } else {
                    println!("Path does not exist: {}", path);
                }
                return Ok(());
            },
            Some(("remove-path", sub_m)) => {
                let path = sub_m.get_one::<String>("path").unwrap();
                config.watch_paths.retain(|p| p != path);
                config.save()?;
                println!("Removed path: {}", path);
                return Ok(());
            },
            Some(("list-paths", _)) => {
                println!("Monitored paths:");
                for path in &config.watch_paths {
                    println!("  {}", path);
                }
                return Ok(());
            },
            Some(("set-url", sub_m)) => {
                let url = sub_m.get_one::<String>("url").unwrap();
                config.siem_url = url.to_string();
                config.save()?;
                println!("SIEM URL updated to: {}", url);
                return Ok(());
            },
            _ => {}
        }
    }

    // Regular agent startup
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