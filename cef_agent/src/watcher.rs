use notify::{Watcher, RecursiveMode};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use std::path::Path;
use crate::error::AgentError;
use crate::api::ApiClient;
use crate::config::AgentConfig;

pub struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::Event>,
    api_client: ApiClient,
}

impl FileWatcher {
    pub fn new(config: AgentConfig) -> Result<Self, AgentError> {
        let (tx, rx) = channel();
        let watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => tx.send(event).unwrap(),
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }).map_err(AgentError::NotifyError)?;

        Ok(FileWatcher {
            watcher,
            receiver: rx,
            api_client: ApiClient::with_config(config),
        })
    }

    pub fn watch_paths(&mut self, paths: &[String]) -> Result<(), AgentError> {
        for path in paths {
            if !Path::new(path).exists() {
                eprintln!("Path does not exist: {}", path);
                continue;
            }
            self.watcher.watch(Path::new(path), RecursiveMode::NonRecursive)
                .map_err(AgentError::NotifyError)?;
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<(), AgentError> {
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(5 * 60));

        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    if let Err(e) = self.api_client.send_heartbeat().await {
                        eprintln!("Failed to send heartbeat: {}", e);
                    }
                }
                _ = async {
                    match self.receiver.recv_timeout(Duration::from_secs(1)) {
                        Ok(event) => {
                            if event.kind.is_modify() || event.kind.is_create() {
                                for path in event.paths {
                                    if path.extension().unwrap_or_default() == "log" {
                                        println!("Change detected in: {:?}", path);
                                        match self.api_client.upload_log(path).await {
                                            Ok(_) => println!("Successfully uploaded log file"),
                                            Err(e) => eprintln!("Failed to upload log file: {}", e),
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            match e {
                                RecvTimeoutError::Timeout => (),
                                RecvTimeoutError::Disconnected => {
                                    eprintln!("Watch error: channel disconnected");
                                    return Err(AgentError::ValidationError("Channel disconnected".to_string()));
                                }
                            }
                        }
                    }
                    Ok::<(), AgentError>(())
                } => {}
            }
        }
    }
}