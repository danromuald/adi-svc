use thiserror::Error;

/// Domain-level errors
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid document format: {0}")]
    InvalidDocumentFormat(String),
    
    #[error("Document too large: {size} bytes (max: {max} bytes)")]
    DocumentTooLarge { size: usize, max: usize },
    
    #[error("Unsupported document type: {0}")]
    UnsupportedDocumentType(String),
    
    #[error("Invalid model type: {0}")]
    InvalidModelType(String),
    
    #[error("Invalid locale: {0}")]
    InvalidLocale(String),
    
    #[error("Invalid page range: {0}")]
    InvalidPageRange(String),
    
    #[error("Document validation failed: {0}")]
    ValidationError(String),
}

pub type DomainResult<T> = Result<T, DomainError>;

