// Re-export protobuf generated code
pub mod id_generator {
    tonic::include_proto!("id_generator.v1");
}

pub mod user {
    tonic::include_proto!("user.v1");
}

// Common response types for HTTP APIs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub timestamp: i64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "Success".to_string(),
            data: Some(data),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn error(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {}

impl<T> Default for ApiResponse<T> {
    fn default() -> Self {
        Self::error(500, "Internal Server Error".to_string())
    }
}
