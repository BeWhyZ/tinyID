/*
 * @Author: AI Assistant
 * @Date: 2025-01-28
 * @Description: Axum 错误处理最佳实践
 */

use axum::{
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Json, Response as AxumResponse},
};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;
use tracing::error;

use super::response::{ErrCode, Response};

// ====================================
// 1. 自定义错误类型
// ====================================

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("验证错误: {message}")]
    Validation { message: String },

    #[error("认证失败: {0}")]
    Authentication(String),

    #[error("权限不足: {0}")]
    Authorization(String),

    #[error("资源未找到: {resource}")]
    NotFound { resource: String },

    #[error("资源冲突: {message}")]
    Conflict { message: String },

    #[error("外部服务错误: {service}: {message}")]
    ExternalService { service: String, message: String },

    #[error("配置错误: {0}")]
    Config(String),

    #[error("内部服务器错误: {0}")]
    Internal(String),

    #[error("请求过于频繁")]
    RateLimit,

    #[error("请求超时")]
    Timeout,

    #[error("请求体过大")]
    PayloadTooLarge,
}

impl ApiError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication(message.into())
    }

    pub fn authorization(message: impl Into<String>) -> Self {
        Self::Authorization(message.into())
    }
}

// ====================================
// 2. 错误到响应的转换
// ====================================

impl IntoResponse for ApiError {
    fn into_response(self) -> AxumResponse {
        let (status_code, err_code, message) = match self {
            ApiError::Database(ref e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrCode::DatabaseError,
                    "数据库操作失败".to_string(),
                )
            }
            ApiError::Validation { ref message } => (
                StatusCode::BAD_REQUEST,
                ErrCode::ValidationError,
                message.clone(),
            ),
            ApiError::Authentication(ref message) => (
                StatusCode::UNAUTHORIZED,
                ErrCode::AuthenticationError,
                message.clone(),
            ),
            ApiError::Authorization(ref message) => (
                StatusCode::FORBIDDEN,
                ErrCode::AuthorizationError,
                message.clone(),
            ),
            ApiError::NotFound { ref resource } => (
                StatusCode::NOT_FOUND,
                ErrCode::NotFound,
                format!("{}不存在", resource),
            ),
            ApiError::Conflict { ref message } => {
                (StatusCode::CONFLICT, ErrCode::Conflict, message.clone())
            }
            ApiError::ExternalService {
                ref service,
                ref message,
            } => {
                error!("External service error - {}: {}", service, message);
                (
                    StatusCode::BAD_GATEWAY,
                    ErrCode::ExternalServiceError,
                    format!("外部服务{}调用失败", service),
                )
            }
            ApiError::Config(ref message) => {
                error!("Config error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrCode::ConfigError,
                    "系统配置错误".to_string(),
                )
            }
            ApiError::Internal(ref message) => {
                error!("Internal error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrCode::InternalServerError,
                    "内部服务器错误".to_string(),
                )
            }
            ApiError::RateLimit => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrCode::RateLimitError,
                "请求过于频繁，请稍后再试".to_string(),
            ),
            ApiError::Timeout => (
                StatusCode::REQUEST_TIMEOUT,
                ErrCode::RequestTimeout,
                "请求超时".to_string(),
            ),
            ApiError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                ErrCode::PayloadTooLarge,
                "请求体过大".to_string(),
            ),
        };

        let response = Response::<()>::failed(err_code, Some(message));
        (status_code, Json(response)).into_response()
    }
}

// ====================================
// 3. Axum 内置错误处理
// ====================================

pub async fn handle_json_rejection(rejection: JsonRejection) -> impl IntoResponse {
    let (status, message) = match rejection {
        JsonRejection::JsonDataError(err) => {
            (StatusCode::BAD_REQUEST, format!("JSON 数据错误: {}", err))
        }
        JsonRejection::JsonSyntaxError(err) => {
            (StatusCode::BAD_REQUEST, format!("JSON 语法错误: {}", err))
        }
        JsonRejection::MissingJsonContentType(_) => (
            StatusCode::BAD_REQUEST,
            "缺少 Content-Type: application/json 头".to_string(),
        ),
        JsonRejection::BytesRejection(err) => {
            (StatusCode::BAD_REQUEST, format!("请求体读取错误: {}", err))
        }
        _ => (StatusCode::BAD_REQUEST, "JSON 请求处理失败".to_string()),
    };

    let response = Response::<()>::failed(ErrCode::BadRequest, Some(message));
    (status, Json(response))
}

pub async fn handle_path_rejection(rejection: PathRejection) -> impl IntoResponse {
    let message = match rejection {
        PathRejection::FailedToDeserializePathParams(err) => {
            format!("路径参数解析失败: {}", err)
        }
        PathRejection::MissingPathParams(err) => {
            format!("缺少路径参数: {}", err)
        }
        _ => "路径参数错误".to_string(),
    };

    let response = Response::<()>::failed(ErrCode::BadRequest, Some(message));
    (StatusCode::BAD_REQUEST, Json(response))
}

pub async fn handle_query_rejection(rejection: QueryRejection) -> impl IntoResponse {
    let message = match rejection {
        QueryRejection::FailedToDeserializeQueryString(err) => {
            format!("查询参数解析失败: {}", err)
        }
        _ => "查询参数错误".to_string(),
    };

    let response = Response::<()>::failed(ErrCode::BadRequest, Some(message));
    (StatusCode::BAD_REQUEST, Json(response))
}

// ====================================
// 4. 结果类型别名
// ====================================

pub type ApiResult<T> = Result<T, ApiError>;

// ====================================
// 5. 验证辅助函数
// ====================================

pub struct Validator;

impl Validator {
    pub fn validate_email(email: &str) -> ApiResult<()> {
        if email.is_empty() {
            return Err(ApiError::validation("邮箱不能为空"));
        }
        if !email.contains('@') || !email.contains('.') {
            return Err(ApiError::validation("邮箱格式不正确"));
        }
        Ok(())
    }

    pub fn validate_username(username: &str) -> ApiResult<()> {
        if username.is_empty() {
            return Err(ApiError::validation("用户名不能为空"));
        }
        if username.len() < 3 {
            return Err(ApiError::validation("用户名长度不能少于3位"));
        }
        if username.len() > 20 {
            return Err(ApiError::validation("用户名长度不能超过20位"));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ApiError::validation("用户名只能包含字母、数字和下划线"));
        }
        Ok(())
    }

    pub fn validate_password(password: &str) -> ApiResult<()> {
        if password.is_empty() {
            return Err(ApiError::validation("密码不能为空"));
        }
        if password.len() < 6 {
            return Err(ApiError::validation("密码长度不能少于6位"));
        }
        if password.len() > 50 {
            return Err(ApiError::validation("密码长度不能超过50位"));
        }
        Ok(())
    }

    pub fn validate_id(id: u64, resource: &str) -> ApiResult<()> {
        if id == 0 {
            return Err(ApiError::validation(format!("{}ID不能为0", resource)));
        }
        Ok(())
    }

    pub fn validate_page_params(page: Option<u32>, size: Option<u32>) -> ApiResult<(u32, u32)> {
        let page = page.unwrap_or(1);
        let size = size.unwrap_or(10);

        if page == 0 {
            return Err(ApiError::validation("页码必须大于0"));
        }
        if size == 0 {
            return Err(ApiError::validation("每页大小必须大于0"));
        }
        if size > 100 {
            return Err(ApiError::validation("每页大小不能超过100"));
        }

        Ok((page, size))
    }
}

// ====================================
// 6. 批量验证
// ====================================

#[derive(Debug)]
pub struct ValidationErrors {
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        let field = field.into();
        let message = message.into();
        self.errors
            .entry(field)
            .or_insert_with(Vec::new)
            .push(message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn into_api_error(self) -> ApiError {
        ApiError::Validation {
            message: "数据验证失败".to_string(),
        }
    }
}

impl IntoResponse for ValidationErrors {
    fn into_response(self) -> AxumResponse {
        let response =
            Response::failed(ErrCode::ValidationError, Some("数据验证失败")).set_data(self.errors);
        (StatusCode::BAD_REQUEST, Json(response)).into_response()
    }
}

// ====================================
// 7. 使用示例
// ====================================

use super::json_response_examples::{CreateUserRequest, UserDto};

pub async fn create_user_with_validation(
    Json(request): Json<CreateUserRequest>,
) -> ApiResult<Json<Response<UserDto>>> {
    // 单个验证
    Validator::validate_username(&request.username)?;
    Validator::validate_email(&request.email)?;
    Validator::validate_password(&request.password)?;

    // 模拟创建用户
    let user = UserDto {
        id: 12345,
        username: request.username,
        email: request.email,
        active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(Response::success(Some(user))))
}

pub async fn create_user_with_batch_validation(
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<Response<UserDto>>, ValidationErrors> {
    let mut errors = ValidationErrors::new();

    // 批量验证
    if let Err(e) = Validator::validate_username(&request.username) {
        errors.add_error("username", e.to_string());
    }

    if let Err(e) = Validator::validate_email(&request.email) {
        errors.add_error("email", e.to_string());
    }

    if let Err(e) = Validator::validate_password(&request.password) {
        errors.add_error("password", e.to_string());
    }

    // 额外的业务验证
    if request.username.to_lowercase() == "admin" {
        errors.add_error("username", "用户名 'admin' 是保留字段");
    }

    if errors.has_errors() {
        return Err(errors);
    }

    // 模拟创建用户
    let user = UserDto {
        id: 12345,
        username: request.username,
        email: request.email,
        active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(Response::success(Some(user))))
}

// ====================================
// 8. 中间件错误处理
// ====================================

pub async fn global_error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
) -> impl IntoResponse {
    error!("Unhandled error: {}", err);

    let response = Response::<()>::failed(
        ErrCode::InternalServerError,
        Some("服务器内部错误，请稍后重试"),
    );

    (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_email() {
        assert!(Validator::validate_email("user@example.com").is_ok());
        assert!(Validator::validate_email("").is_err());
        assert!(Validator::validate_email("invalid-email").is_err());
        assert!(Validator::validate_email("user@").is_err());
        assert!(Validator::validate_email("@example.com").is_err());
    }

    #[test]
    fn test_validator_username() {
        assert!(Validator::validate_username("valid_user").is_ok());
        assert!(Validator::validate_username("user123").is_ok());
        assert!(Validator::validate_username("").is_err());
        assert!(Validator::validate_username("ab").is_err());
        assert!(Validator::validate_username("a".repeat(21).as_str()).is_err());
        assert!(Validator::validate_username("user-name").is_err());
    }

    #[test]
    fn test_validator_password() {
        assert!(Validator::validate_password("password123").is_ok());
        assert!(Validator::validate_password("").is_err());
        assert!(Validator::validate_password("12345").is_err());
        assert!(Validator::validate_password(&"a".repeat(51)).is_err());
    }

    #[test]
    fn test_validator_page_params() {
        assert_eq!(
            Validator::validate_page_params(Some(1), Some(10)).unwrap(),
            (1, 10)
        );
        assert_eq!(
            Validator::validate_page_params(None, None).unwrap(),
            (1, 10)
        );
        assert!(Validator::validate_page_params(Some(0), Some(10)).is_err());
        assert!(Validator::validate_page_params(Some(1), Some(0)).is_err());
        assert!(Validator::validate_page_params(Some(1), Some(101)).is_err());
    }

    #[test]
    fn test_validation_errors() {
        let mut errors = ValidationErrors::new();
        assert!(!errors.has_errors());

        errors.add_error("field1", "error1");
        errors.add_error("field1", "error2");
        errors.add_error("field2", "error3");

        assert!(errors.has_errors());
        assert_eq!(errors.errors.get("field1").unwrap().len(), 2);
        assert_eq!(errors.errors.get("field2").unwrap().len(), 1);
    }
}
