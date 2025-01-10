use notify::{Watcher, RecursiveMode};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use std::path::Path;
use crate::error::AgentError;

pub struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::Event>,
}

impl FileWatcher {
    pub fn new() -> Result<Self, AgentError> {
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

    pub fn run(&self) -> Result<(), AgentError> {
        loop {
            match self.receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => {
                    if event.kind.is_modify() || event.kind.is_create() {
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
}