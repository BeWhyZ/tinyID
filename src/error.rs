use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TinyIdError {
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Invalid worker ID: {0}")]
    InvalidWorkerId(u32),

    #[error("Invalid datacenter ID: {0}")]
    InvalidDatacenterId(u32),

    #[error("configuration error:{0}")]
    ConfigError(String),

    #[error("Clock moved backwards by {0}ms")]
    ClockMovedBackwards(u64),

    #[error("Server error: {0}")]
    ServerError(String),
}

impl From<std::io::Error> for TinyIdError {
    fn from(err: std::io::Error) -> Self {
        TinyIdError::InternalError(err.to_string())
    }
}
