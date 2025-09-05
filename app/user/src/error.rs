use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(u64),

    #[error("Invalid user data: {0}")]
    InvalidData(String),

    #[error("User already exists: {0}")]
    AlreadyExists(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<UserError> for tonic::Status {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound(_) => tonic::Status::not_found(err.to_string()),
            UserError::InvalidData(_) => tonic::Status::invalid_argument(err.to_string()),
            UserError::AlreadyExists(_) => tonic::Status::already_exists(err.to_string()),
            UserError::DatabaseError(_) => tonic::Status::internal(err.to_string()),
            UserError::InternalError(_) => tonic::Status::internal(err.to_string()),
        }
    }
}
