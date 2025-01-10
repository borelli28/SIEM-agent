use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AgentError {
    ApiError(Box<dyn Error>),
    NotifyError(notify::Error),
    IoError(std::io::Error),
    ValidationError(String),
    UploadError(String),
}

impl From<std::io::Error> for AgentError {
    fn from(err: std::io::Error) -> Self {
        AgentError::IoError(err)
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::ApiError(e) => write!(f, "API error: {}", e),
            AgentError::NotifyError(e) => write!(f, "Notify error: {}", e),
            AgentError::IoError(e) => write!(f, "IO error: {}", e),
            AgentError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AgentError::UploadError(e) => write!(f, "Upload error: {}", e),
        }
    }
}

impl Error for AgentError {}

impl From<notify::Error> for AgentError {
    fn from(err: notify::Error) -> Self {
        AgentError::NotifyError(err)
    }
}

impl From<reqwest::Error> for AgentError {
    fn from(err: reqwest::Error) -> Self {
        AgentError::ApiError(Box::new(err))
    }
}

impl From<String> for AgentError {
    fn from(err: String) -> Self {
        AgentError::ValidationError(err)
    }
}

impl From<serde_json::Error> for AgentError {
    fn from(err: serde_json::Error) -> Self {
        AgentError::ApiError(Box::new(err))
    }
}