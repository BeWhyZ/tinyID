use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TinyIdError {
    #[error("ID generation failed: {0}")]
    IdGenerationFailed(String),

    #[error("User service error: {0}")]
    UserServiceError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Clock moved backwards by {0}ms")]
    ClockMovedBackwards(u64),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Invalid worker ID: {0}")]
    InvalidWorkerId(u32),

    #[error("Invalid datacenter ID: {0}")]
    InvalidDatacenterId(u32),

    #[error("configuration error:{0}")]
    ConfigError(String),

    #[error("Server error: {0}")]
    ServerError(String),
}

impl From<TinyIdError> for axum::response::Response<axum::body::Body> {
    fn from(err: TinyIdError) -> Self {
        use axum::response::{IntoResponse, Json};
        use shared::proto::ApiResponse;

        let (status, code, message) = match err {
            TinyIdError::InvalidRequest(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                err.to_string(),
            ),
            TinyIdError::InvalidDatacenterId(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                err.to_string(),
            ),
            TinyIdError::ConfigError(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                err.to_string(),
            ),
            TinyIdError::ServerError(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                err.to_string(),
            ),

            TinyIdError::InvalidWorkerId(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                err.to_string(),
            ),
            TinyIdError::ClockMovedBackwards(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                err.to_string(),
            ),
            TinyIdError::IdGenerationFailed(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                err.to_string(),
            ),
            TinyIdError::UserServiceError(_) => {
                (axum::http::StatusCode::BAD_GATEWAY, 5002, err.to_string())
            }
            TinyIdError::InternalError(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5000,
                err.to_string(),
            ),
        };

        let response: ApiResponse<()> = ApiResponse::error(code, message);
        (status, Json(response)).into_response()
    }
}

impl From<std::io::Error> for TinyIdError {
    fn from(err: std::io::Error) -> Self {
        TinyIdError::InternalError(err.to_string())
    }
}
