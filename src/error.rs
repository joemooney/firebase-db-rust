use thiserror::Error;

#[derive(Error, Debug)]
pub enum FirebaseError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("JSON serialization/deserialization failed: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("Authentication failed: {0}")]
    AuthError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Database operation failed: {0}")]
    DatabaseError(String),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, FirebaseError>;