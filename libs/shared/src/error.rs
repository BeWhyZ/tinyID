use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SharedError {
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}
