use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Intent pattern mismatch: {0}")]
    PatternMismatch(String),
    
    #[error("Service already registered: {0}")]
    ServiceAlreadyExists(String),
    
    #[error("Communication error: {0}")]
    Communication(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    Connection(String),
    
    #[error("Query execution error: {0}")]
    Query(String),
    
    #[error("Data serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}