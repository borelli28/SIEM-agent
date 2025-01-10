mod api;
mod error;
mod config;
mod cli;
mod prompt;
mod registration;
mod watcher;

use crate::error::AgentError;
use crate::config::AgentConfig;

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    let matches = cli::create_cli().get_matches();

    // Handle configuration
    if let Some(config_matches) = matches.subcommand_matches("config") {
        let mut config = AgentConfig::load()
            .ok_or(AgentError::ValidationError("No configuration found".to_string()))?;

        match config_matches.subcommand() {
            Some(("add-path", sub_m)) => {
                let path = sub_m.get_one::<String>("path").unwrap();
                if std::path::Path::new(path).exists() {
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

    // Regular startup
    let config = match AgentConfig::load() {
        Some(config) => config,
        None => {
            println!("No configuration found. Let's set up the agent.");

            let host_id = prompt::prompt("Enter host ID: ")?;
            let account_id = prompt::prompt("Enter account ID: ")?;
            let hostname = prompt::prompt("Enter hostname: ")?;
            let siem_url = prompt::prompt("Enter SIEM URL (default: http://localhost:4200): ")?;
            let siem_url = if siem_url.is_empty() {
                "http://localhost:4200".to_string()
            } else {
                siem_url
            };

            println!("Now let's set up the paths to monitor.");
            let watch_paths = prompt::prompt_watch_paths()?;

            let response = registration::register_agent(
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

    // Initialize and run file watcher
    let mut file_watcher = watcher::FileWatcher::new()?;
    file_watcher.watch_paths(&config.watch_paths)?;
    file_watcher.run()?;

    Ok(())
}