use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AgentError {
    ApiError(Box<dyn Error>),
    NotifyError(notify::Error),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::ApiError(e) => write!(f, "API error: {}", e),
            AgentError::NotifyError(e) => write!(f, "Notify error: {}", e),
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
        AgentError::ApiError(err.into())
    }
}
