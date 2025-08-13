/*
 * @Author: iwhyz
 * @Date: 2025-01-28
 * @Description: 通用Response结构定义，符合Rust最佳实践
 */

use serde::{Deserialize, Serialize};

use http::StatusCode;

/// 业务错误码枚举，与HTTP状态码对应
///
/// 每个错误码都有对应的HTTP状态码和默认的错误消息
/// 可以用于API响应的统一错误处理
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrCode {
    // 成功状态 (2xx)
    /// 操作成功 - HTTP 200
    Success = 0,

    // 客户端错误 (4xx)
    /// 请求参数错误 - HTTP 400
    BadRequest = 400,
    /// 未授权访问 - HTTP 401  
    Unauthorized = 401,
    /// 权限不足 - HTTP 403
    Forbidden = 403,
    /// 资源未找到 - HTTP 404
    NotFound = 404,
    /// 请求方法不允许 - HTTP 405
    MethodNotAllowed = 405,
    /// 请求超时 - HTTP 408
    RequestTimeout = 408,
    /// 资源冲突 - HTTP 409
    Conflict = 409,
    /// 请求实体过大 - HTTP 413
    PayloadTooLarge = 413,
    /// 请求过于频繁 - HTTP 429
    TooManyRequests = 429,

    // 服务器错误 (5xx)
    /// 内部服务器错误 - HTTP 500
    InternalServerError = 500,
    /// 功能未实现 - HTTP 501
    NotImplemented = 501,
    /// 网关错误 - HTTP 502
    BadGateway = 502,
    /// 服务不可用 - HTTP 503
    ServiceUnavailable = 503,
    /// 网关超时 - HTTP 504
    GatewayTimeout = 504,

    // 业务特定错误码 (1000+)
    /// 参数验证失败
    ValidationError = 1001,
    /// 数据库操作失败
    DatabaseError = 1002,
    /// 外部服务调用失败
    ExternalServiceError = 1003,
    /// 配置错误
    ConfigError = 1004,
    /// 认证失败
    AuthenticationError = 1005,
    /// 授权失败
    AuthorizationError = 1006,
    /// 业务逻辑错误
    BusinessLogicError = 1007,
    /// 数据不一致
    DataInconsistencyError = 1008,
    /// 限流错误
    RateLimitError = 1009,
    /// 缓存错误
    CacheError = 1010,
}

impl ErrCode {
    /// 获取对应的HTTP状态码
    pub fn http_status(&self) -> u16 {
        match *self {
            ErrCode::Success => StatusCode::OK.as_u16(),
            ErrCode::BadRequest => StatusCode::BAD_REQUEST.as_u16(),
            ErrCode::Unauthorized => StatusCode::UNAUTHORIZED.as_u16(),
            ErrCode::Forbidden => StatusCode::FORBIDDEN.as_u16(),
            ErrCode::NotFound => StatusCode::NOT_FOUND.as_u16(),
            ErrCode::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED.as_u16(),
            ErrCode::RequestTimeout => StatusCode::REQUEST_TIMEOUT.as_u16(),
            ErrCode::Conflict => StatusCode::CONFLICT.as_u16(),
            ErrCode::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE.as_u16(),
            ErrCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS.as_u16(),
            ErrCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ErrCode::NotImplemented => StatusCode::NOT_IMPLEMENTED.as_u16(),
            ErrCode::BadGateway => StatusCode::BAD_GATEWAY.as_u16(),
            ErrCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE.as_u16(),
            ErrCode::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT.as_u16(),
            // 业务错误通常映射为400或500
            ErrCode::ValidationError => StatusCode::BAD_REQUEST.as_u16(),
            ErrCode::AuthenticationError => StatusCode::UNAUTHORIZED.as_u16(),
            ErrCode::AuthorizationError => StatusCode::FORBIDDEN.as_u16(),
            ErrCode::BusinessLogicError => StatusCode::BAD_REQUEST.as_u16(),
            ErrCode::DataInconsistencyError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ErrCode::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ErrCode::ExternalServiceError => StatusCode::BAD_GATEWAY.as_u16(),
            ErrCode::ConfigError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ErrCode::RateLimitError => StatusCode::TOO_MANY_REQUESTS.as_u16(),
            ErrCode::CacheError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        }
    }

    /// 获取默认的错误消息
    pub fn default_message(&self) -> &'static str {
        match *self {
            ErrCode::Success => "操作成功",
            ErrCode::BadRequest => "请求参数错误",
            ErrCode::Unauthorized => "未授权访问",
            ErrCode::Forbidden => "权限不足",
            ErrCode::NotFound => "资源未找到",
            ErrCode::MethodNotAllowed => "请求方法不允许",
            ErrCode::RequestTimeout => "请求超时",
            ErrCode::Conflict => "资源冲突",
            ErrCode::PayloadTooLarge => "请求实体过大",
            ErrCode::TooManyRequests => "请求过于频繁",
            ErrCode::InternalServerError => "内部服务器错误",
            ErrCode::NotImplemented => "功能未实现",
            ErrCode::BadGateway => "网关错误",
            ErrCode::ServiceUnavailable => "服务不可用",
            ErrCode::GatewayTimeout => "网关超时",
            ErrCode::ValidationError => "参数验证失败",
            ErrCode::DatabaseError => "数据库操作失败",
            ErrCode::ExternalServiceError => "外部服务调用失败",
            ErrCode::ConfigError => "配置错误",
            ErrCode::AuthenticationError => "认证失败",
            ErrCode::AuthorizationError => "授权失败",
            ErrCode::BusinessLogicError => "业务逻辑错误",
            ErrCode::DataInconsistencyError => "数据不一致",
            ErrCode::RateLimitError => "访问频率超限",
            ErrCode::CacheError => "缓存操作失败",
        }
    }

    /// 判断是否为成功状态
    pub fn is_success(&self) -> bool {
        matches!(*self, ErrCode::Success)
    }

    /// 判断是否为客户端错误 (4xx)
    pub fn is_client_error(&self) -> bool {
        let status = self.http_status();
        status >= 400 && status < 500
    }

    /// 判断是否为服务器错误 (5xx)
    pub fn is_server_error(&self) -> bool {
        let status = self.http_status();
        status >= 500 && status < 600
    }

    /// 判断是否为业务错误 (1000+)
    pub fn is_business_error(&self) -> bool {
        (*self as i32) >= 1000
    }
}

// 实现序列化时使用数值
impl Serialize for ErrCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

// 实现反序列化
impl<'de> Deserialize<'de> for ErrCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let code = i32::deserialize(deserializer)?;
        match code {
            0 => Ok(ErrCode::Success),
            400 => Ok(ErrCode::BadRequest),
            401 => Ok(ErrCode::Unauthorized),
            403 => Ok(ErrCode::Forbidden),
            404 => Ok(ErrCode::NotFound),
            405 => Ok(ErrCode::MethodNotAllowed),
            408 => Ok(ErrCode::RequestTimeout),
            409 => Ok(ErrCode::Conflict),
            413 => Ok(ErrCode::PayloadTooLarge),
            429 => Ok(ErrCode::TooManyRequests),
            500 => Ok(ErrCode::InternalServerError),
            501 => Ok(ErrCode::NotImplemented),
            502 => Ok(ErrCode::BadGateway),
            503 => Ok(ErrCode::ServiceUnavailable),
            504 => Ok(ErrCode::GatewayTimeout),
            1001 => Ok(ErrCode::ValidationError),
            1002 => Ok(ErrCode::DatabaseError),
            1003 => Ok(ErrCode::ExternalServiceError),
            1004 => Ok(ErrCode::ConfigError),
            1005 => Ok(ErrCode::AuthenticationError),
            1006 => Ok(ErrCode::AuthorizationError),
            1007 => Ok(ErrCode::BusinessLogicError),
            1008 => Ok(ErrCode::DataInconsistencyError),
            1009 => Ok(ErrCode::RateLimitError),
            1010 => Ok(ErrCode::CacheError),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown error code: {}",
                code
            ))),
        }
    }
}

// 从HTTP状态码转换
impl From<u16> for ErrCode {
    fn from(status: u16) -> Self {
        match status {
            200 => ErrCode::Success,
            400 => ErrCode::BadRequest,
            401 => ErrCode::Unauthorized,
            403 => ErrCode::Forbidden,
            404 => ErrCode::NotFound,
            405 => ErrCode::MethodNotAllowed,
            408 => ErrCode::RequestTimeout,
            409 => ErrCode::Conflict,
            413 => ErrCode::PayloadTooLarge,
            429 => ErrCode::TooManyRequests,
            500 => ErrCode::InternalServerError,
            501 => ErrCode::NotImplemented,
            502 => ErrCode::BadGateway,
            503 => ErrCode::ServiceUnavailable,
            504 => ErrCode::GatewayTimeout,
            _ => ErrCode::InternalServerError, // 默认映射到500
        }
    }
}

// 显示实现
impl std::fmt::Display for ErrCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T = ()>
where
    T: Serialize,
{
    /// 响应状态码
    pub code: ErrCode,
    /// 响应消息
    pub msg: String,
    /// 引用信息，用于调试和追踪
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> Response<T>
where
    T: Serialize,
{
    /// 创建一个新的Response实例
    pub fn new(code: ErrCode, msg: impl Into<String>) -> Self {
        Self {
            code,
            msg: msg.into(),
            r#ref: None,
            data: None,
        }
    }

    /// 创建带有数据的Response
    pub fn with_data(code: ErrCode, msg: impl Into<String>, data: T) -> Self {
        Self {
            code,
            msg: msg.into(),
            r#ref: None,
            data: Some(data),
        }
    }

    pub fn set_ref(mut self, r#ref: impl Into<String>) -> Self {
        self.r#ref = Some(r#ref.into());
        self
    }

    pub fn set_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    // 成功响应
    pub fn success(data: Option<T>) -> Self {
        if data.is_none() {
            Self::new(ErrCode::Success, ErrCode::Success.default_message())
        } else {
            Self::with_data(
                ErrCode::Success,
                ErrCode::Success.default_message(),
                data.unwrap(),
            )
        }
    }

    // 失败响应
    pub fn failed(code: ErrCode, msg: Option<impl Into<String>>) -> Self {
        if msg.is_none() {
            Self::new(code, code.default_message())
        } else {
            Self::new(code, msg.unwrap())
        }
    }
}

// 为了方便测试，实现PartialEq
impl<T> PartialEq for Response<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
            && self.msg == other.msg
            && self.r#ref == other.r#ref
            && self.data == other.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u64,
        name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct UserData {
        user_id: u32,
        username: String,
        email: String,
        active: bool,
    }

    // ================================
    // ErrCode 测试
    // ================================

    #[test]
    fn test_errcode_http_status_mapping() {
        // 成功状态
        assert_eq!(ErrCode::Success.http_status(), 200);

        // 客户端错误 (4xx)
        assert_eq!(ErrCode::BadRequest.http_status(), 400);
        assert_eq!(ErrCode::Unauthorized.http_status(), 401);
        assert_eq!(ErrCode::Forbidden.http_status(), 403);
        assert_eq!(ErrCode::NotFound.http_status(), 404);
        assert_eq!(ErrCode::MethodNotAllowed.http_status(), 405);
        assert_eq!(ErrCode::RequestTimeout.http_status(), 408);
        assert_eq!(ErrCode::Conflict.http_status(), 409);
        assert_eq!(ErrCode::PayloadTooLarge.http_status(), 413);
        assert_eq!(ErrCode::TooManyRequests.http_status(), 429);

        // 服务器错误 (5xx)
        assert_eq!(ErrCode::InternalServerError.http_status(), 500);
        assert_eq!(ErrCode::NotImplemented.http_status(), 501);
        assert_eq!(ErrCode::BadGateway.http_status(), 502);
        assert_eq!(ErrCode::ServiceUnavailable.http_status(), 503);
        assert_eq!(ErrCode::GatewayTimeout.http_status(), 504);

        // 业务错误映射
        assert_eq!(ErrCode::ValidationError.http_status(), 400);
        assert_eq!(ErrCode::AuthenticationError.http_status(), 401);
        assert_eq!(ErrCode::AuthorizationError.http_status(), 403);
        assert_eq!(ErrCode::BusinessLogicError.http_status(), 400);
        assert_eq!(ErrCode::DatabaseError.http_status(), 500);
        assert_eq!(ErrCode::ExternalServiceError.http_status(), 502);
        assert_eq!(ErrCode::ConfigError.http_status(), 500);
        assert_eq!(ErrCode::DataInconsistencyError.http_status(), 500);
        assert_eq!(ErrCode::RateLimitError.http_status(), 429);
        assert_eq!(ErrCode::CacheError.http_status(), 500);
    }

    #[test]
    fn test_errcode_default_messages() {
        assert_eq!(ErrCode::Success.default_message(), "操作成功");
        assert_eq!(ErrCode::BadRequest.default_message(), "请求参数错误");
        assert_eq!(ErrCode::Unauthorized.default_message(), "未授权访问");
        assert_eq!(ErrCode::Forbidden.default_message(), "权限不足");
        assert_eq!(ErrCode::NotFound.default_message(), "资源未找到");
        assert_eq!(
            ErrCode::InternalServerError.default_message(),
            "内部服务器错误"
        );
        assert_eq!(ErrCode::ValidationError.default_message(), "参数验证失败");
        assert_eq!(ErrCode::DatabaseError.default_message(), "数据库操作失败");
        assert_eq!(ErrCode::AuthenticationError.default_message(), "认证失败");
        assert_eq!(ErrCode::AuthorizationError.default_message(), "授权失败");
        assert_eq!(
            ErrCode::BusinessLogicError.default_message(),
            "业务逻辑错误"
        );
        assert_eq!(
            ErrCode::ExternalServiceError.default_message(),
            "外部服务调用失败"
        );
        assert_eq!(ErrCode::ConfigError.default_message(), "配置错误");
        assert_eq!(
            ErrCode::DataInconsistencyError.default_message(),
            "数据不一致"
        );
        assert_eq!(ErrCode::RateLimitError.default_message(), "访问频率超限");
        assert_eq!(ErrCode::CacheError.default_message(), "缓存操作失败");
    }

    #[test]
    fn test_errcode_status_checks() {
        // 成功状态检查
        assert!(ErrCode::Success.is_success());
        assert!(!ErrCode::BadRequest.is_success());
        assert!(!ErrCode::InternalServerError.is_success());
        assert!(!ErrCode::ValidationError.is_success());

        // 客户端错误检查
        assert!(ErrCode::BadRequest.is_client_error());
        assert!(ErrCode::Unauthorized.is_client_error());
        assert!(ErrCode::NotFound.is_client_error());
        assert!(ErrCode::ValidationError.is_client_error());
        assert!(!ErrCode::Success.is_client_error());
        assert!(!ErrCode::InternalServerError.is_client_error());

        // 服务器错误检查
        assert!(ErrCode::InternalServerError.is_server_error());
        assert!(ErrCode::BadGateway.is_server_error());
        assert!(ErrCode::DatabaseError.is_server_error());
        assert!(ErrCode::ConfigError.is_server_error());
        assert!(!ErrCode::Success.is_server_error());
        assert!(!ErrCode::BadRequest.is_server_error());

        // 业务错误检查
        assert!(ErrCode::ValidationError.is_business_error());
        assert!(ErrCode::DatabaseError.is_business_error());
        assert!(ErrCode::AuthenticationError.is_business_error());
        assert!(ErrCode::BusinessLogicError.is_business_error());
        assert!(!ErrCode::Success.is_business_error());
        assert!(!ErrCode::BadRequest.is_business_error());
        assert!(!ErrCode::InternalServerError.is_business_error());
    }

    #[test]
    fn test_errcode_serialization() {
        // 测试序列化
        let test_cases = vec![
            (ErrCode::Success, "0"),
            (ErrCode::BadRequest, "400"),
            (ErrCode::NotFound, "404"),
            (ErrCode::InternalServerError, "500"),
            (ErrCode::ValidationError, "1001"),
            (ErrCode::DatabaseError, "1002"),
        ];

        for (code, expected_json) in test_cases {
            let json = serde_json::to_string(&code).unwrap();
            assert_eq!(json, expected_json);

            // 测试反序列化
            let deserialized: ErrCode = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, code);
        }
    }

    #[test]
    fn test_errcode_from_http_status() {
        assert_eq!(ErrCode::from(200), ErrCode::Success);
        assert_eq!(ErrCode::from(400), ErrCode::BadRequest);
        assert_eq!(ErrCode::from(401), ErrCode::Unauthorized);
        assert_eq!(ErrCode::from(404), ErrCode::NotFound);
        assert_eq!(ErrCode::from(500), ErrCode::InternalServerError);

        // 未知状态码默认映射到500
        assert_eq!(ErrCode::from(418), ErrCode::InternalServerError); // I'm a teapot
        assert_eq!(ErrCode::from(999), ErrCode::InternalServerError);
    }

    #[test]
    fn test_errcode_display() {
        assert_eq!(ErrCode::Success.to_string(), "0");
        assert_eq!(ErrCode::NotFound.to_string(), "404");
        assert_eq!(ErrCode::ValidationError.to_string(), "1001");
    }

    #[test]
    fn test_errcode_deserialization_invalid() {
        let invalid_json = "99999";
        let result: Result<ErrCode, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    // ================================
    // Response 基础测试
    // ================================

    #[test]
    fn test_response_new() {
        let response = Response::<()>::new(ErrCode::Success, "操作成功");
        assert_eq!(response.code, ErrCode::Success);
        assert_eq!(response.msg, "操作成功");
        assert!(response.r#ref.is_none());
        assert!(response.data.is_none());
    }

    #[test]
    fn test_response_with_data() {
        let data = TestData {
            id: 1,
            name: "测试数据".to_string(),
        };
        let response = Response::with_data(ErrCode::Success, "获取成功", data.clone());

        assert_eq!(response.code, ErrCode::Success);
        assert_eq!(response.msg, "获取成功");
        assert!(response.r#ref.is_none());
        assert_eq!(response.data, Some(data));
    }

    #[test]
    fn test_response_set_ref() {
        let response = Response::<()>::new(ErrCode::Success, "成功").set_ref("req-123456");

        assert_eq!(response.r#ref, Some("req-123456".to_string()));
    }

    #[test]
    fn test_response_set_data() {
        let data = TestData {
            id: 42,
            name: "动态设置".to_string(),
        };
        let response = Response::new(ErrCode::Success, "成功").set_data(data.clone());

        assert_eq!(response.data, Some(data));
    }

    #[test]
    fn test_response_chaining() {
        let data = UserData {
            user_id: 100,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            active: true,
        };

        let response = Response::new(ErrCode::Success, "用户创建成功")
            .set_ref("req-create-user-001")
            .set_data(data.clone());

        assert_eq!(response.code, ErrCode::Success);
        assert_eq!(response.msg, "用户创建成功");
        assert_eq!(response.r#ref, Some("req-create-user-001".to_string()));
        assert_eq!(response.data, Some(data));
    }

    // ================================
    // Response 便利方法测试
    // ================================

    #[test]
    fn test_response_success() {
        // 无数据成功响应
        let response = Response::<()>::success(None);
        assert_eq!(response.code, ErrCode::Success);
        assert!(response.data.is_none());

        // 带数据成功响应
        let data = TestData {
            id: 1,
            name: "成功数据".to_string(),
        };
        let response = Response::success(Some(data.clone()));
        assert_eq!(response.code, ErrCode::Success);
        assert_eq!(response.data, Some(data));
    }

    #[test]
    fn test_response_failed() {
        // 使用默认错误消息
        let response = Response::<()>::failed(ErrCode::ValidationError, None::<String>);
        assert_eq!(response.code, ErrCode::ValidationError);
        assert_eq!(response.msg, "参数验证失败");
        assert!(response.data.is_none());

        // 使用自定义错误消息
        let response = Response::<()>::failed(ErrCode::NotFound, Some("用户不存在"));
        assert_eq!(response.code, ErrCode::NotFound);
        assert_eq!(response.msg, "用户不存在");
        assert!(response.data.is_none());
    }

    // ================================
    // Response 序列化测试
    // ================================

    #[test]
    fn test_response_serialization_minimal() {
        let response = Response::<()>::new(ErrCode::Success, "成功");
        let json = serde_json::to_string(&response).unwrap();

        // 验证JSON结构
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["code"], 0);
        assert_eq!(value["msg"], "成功");
        assert!(value["ref"].is_null());
        assert!(value["data"].is_null());
    }

    #[test]
    fn test_response_serialization_with_data() {
        let data = TestData {
            id: 123,
            name: "序列化测试".to_string(),
        };
        let response = Response::with_data(ErrCode::Success, "数据获取成功", data);

        let json = serde_json::to_string(&response).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["code"], 0);
        assert_eq!(value["msg"], "数据获取成功");
        assert_eq!(value["data"]["id"], 123);
        assert_eq!(value["data"]["name"], "序列化测试");
    }

    #[test]
    fn test_response_serialization_with_ref() {
        let response = Response::<()>::new(ErrCode::NotFound, "资源未找到").set_ref("req-404-001");

        let json = serde_json::to_string(&response).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["code"], 404);
        assert_eq!(value["msg"], "资源未找到");
        assert_eq!(value["ref"], "req-404-001");
        assert!(value["data"].is_null());
    }

    #[test]
    fn test_response_serialization_complete() {
        let data = UserData {
            user_id: 456,
            username: "fulltest".to_string(),
            email: "full@test.com".to_string(),
            active: false,
        };

        let response =
            Response::with_data(ErrCode::Success, "完整数据", data).set_ref("req-full-test");

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Response<UserData> = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
        assert_eq!(deserialized.code, ErrCode::Success);
        assert_eq!(deserialized.msg, "完整数据");
        assert_eq!(deserialized.r#ref, Some("req-full-test".to_string()));
        assert!(deserialized.data.is_some());
        assert_eq!(deserialized.data.unwrap().user_id, 456);
    }

    // ================================
    // 边界情况和错误处理测试
    // ================================

    #[test]
    fn test_response_empty_strings() {
        let response = Response::<()>::new(ErrCode::Success, "");
        assert_eq!(response.msg, "");

        let response = response.set_ref("");
        assert_eq!(response.r#ref, Some("".to_string()));
    }

    #[test]
    fn test_response_unicode_strings() {
        let response = Response::<()>::new(ErrCode::Success, "操作成功 ✅ 🎉");
        assert_eq!(response.msg, "操作成功 ✅ 🎉");

        let response = response.set_ref("请求-123-🔥");
        assert_eq!(response.r#ref, Some("请求-123-🔥".to_string()));
    }

    #[test]
    fn test_response_large_data() {
        let large_data: Vec<TestData> = (0..1000)
            .map(|i| TestData {
                id: i,
                name: format!("Item {}", i),
            })
            .collect();

        let response = Response::with_data(ErrCode::Success, "大量数据", large_data.clone());

        assert_eq!(response.code, ErrCode::Success);
        assert_eq!(response.data.as_ref().unwrap().len(), 1000);

        // 测试序列化大数据
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.len() > 10000); // 确保序列化了大量数据
    }

    // ================================
    // 实际使用场景测试
    // ================================

    #[test]
    fn test_api_success_scenarios() {
        // 场景1: 用户登录成功
        let user = UserData {
            user_id: 1001,
            username: "john_doe".to_string(),
            email: "john@example.com".to_string(),
            active: true,
        };
        let response = Response::success(Some(user));
        assert!(response.code.is_success());
        assert!(response.data.is_some());

        // 场景2: 简单操作成功
        let response = Response::<()>::success(None);
        assert_eq!(response.code, ErrCode::Success);
        assert!(response.data.is_none());
    }

    #[test]
    fn test_api_error_scenarios() {
        // 场景1: 验证失败
        let response = Response::<()>::failed(ErrCode::ValidationError, Some("邮箱格式不正确"));
        assert!(response.code.is_client_error());
        assert!(response.code.is_business_error());
        assert_eq!(response.msg, "邮箱格式不正确");

        // 场景2: 资源未找到
        let response =
            Response::<()>::failed(ErrCode::NotFound, None::<String>).set_ref("user-404");
        assert_eq!(response.code, ErrCode::NotFound);
        assert_eq!(response.msg, "资源未找到");
        assert_eq!(response.r#ref, Some("user-404".to_string()));

        // 场景3: 服务器错误
        let response = Response::<()>::failed(ErrCode::DatabaseError, Some("数据库连接超时"));
        assert!(response.code.is_server_error());
        assert!(response.code.is_business_error());
    }

    #[test]
    fn test_error_code_categorization() {
        let client_errors = vec![
            ErrCode::BadRequest,
            ErrCode::Unauthorized,
            ErrCode::Forbidden,
            ErrCode::NotFound,
            ErrCode::ValidationError,
            ErrCode::AuthenticationError,
            ErrCode::AuthorizationError,
            ErrCode::BusinessLogicError,
        ];

        let server_errors = vec![
            ErrCode::InternalServerError,
            ErrCode::BadGateway,
            ErrCode::ServiceUnavailable,
            ErrCode::DatabaseError,
            ErrCode::ConfigError,
            ErrCode::DataInconsistencyError,
            ErrCode::CacheError,
        ];

        for error in client_errors {
            assert!(
                error.is_client_error(),
                "Expected {:?} to be client error",
                error
            );
            assert!(
                !error.is_server_error(),
                "Expected {:?} to not be server error",
                error
            );
        }

        for error in server_errors {
            assert!(
                error.is_server_error(),
                "Expected {:?} to be server error",
                error
            );
            assert!(
                !error.is_client_error(),
                "Expected {:?} to not be client error",
                error
            );
        }
    }

    // ================================
    // 性能相关测试
    // ================================

    #[test]
    fn test_response_clone() {
        let data = TestData {
            id: 999,
            name: "克隆测试".to_string(),
        };
        let original =
            Response::with_data(ErrCode::Success, "原始数据", data.clone()).set_ref("clone-test");

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.data.unwrap().id, 999);
    }

    #[test]
    fn test_multiple_set_operations() {
        let mut response = Response::<TestData>::new(ErrCode::Success, "初始消息");

        // 多次设置引用
        response = response.set_ref("ref-1");
        response = response.set_ref("ref-2");
        response = response.set_ref("ref-final");

        assert_eq!(response.r#ref, Some("ref-final".to_string()));

        // 设置数据
        let data1 = TestData {
            id: 1,
            name: "数据1".to_string(),
        };
        let data2 = TestData {
            id: 2,
            name: "数据2".to_string(),
        };

        response = response.set_data(data1);
        response = response.set_data(data2.clone());

        assert_eq!(response.data, Some(data2));
    }
}
