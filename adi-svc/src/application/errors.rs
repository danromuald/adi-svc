use thiserror::Error;
use crate::domain::DomainError;

/// Application-level errors
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Azure service error: {0}")]
    AzureService(String),
    
    #[error("Operation not found: {0}")]
    OperationNotFound(String),
    
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for ApplicationError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

