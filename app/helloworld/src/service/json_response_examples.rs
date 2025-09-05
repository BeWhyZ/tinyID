/*
 * @Author: AI Assistant
 * @Date: 2025-01-28
 * @Description: Axum JSON 响应最佳实践示例
 */

use axum::{
    extract::{Path, Query},
    http::{StatusCode, HeaderMap},
    response::{Json, Response as AxumResponse, IntoResponse},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::response::{ErrCode, Response};

// ====================================
// 1. 数据传输对象 (DTOs)
// ====================================

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub size: u32,
    pub pages: u32,
}

// ====================================
// 2. 自定义响应类型
// ====================================

/// 带 HTTP 状态码的 JSON 响应
pub struct JsonResponse<T>(pub StatusCode, pub Json<Response<T>>)
where
    T: Serialize;

impl<T> IntoResponse for JsonResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> AxumResponse {
        (self.0, self.1).into_response()
    }
}

impl<T> JsonResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Self(StatusCode::OK, Json(Response::success(Some(data))))
    }

    pub fn created(data: T) -> Self {
        Self(StatusCode::CREATED, Json(Response::success(Some(data))))
    }

    pub fn no_content() -> JsonResponse<()> {
        JsonResponse(StatusCode::NO_CONTENT, Json(Response::success(None)))
    }

    pub fn bad_request(message: &str) -> JsonResponse<()> {
        JsonResponse(
            StatusCode::BAD_REQUEST,
            Json(Response::failed(ErrCode::BadRequest, Some(message))),
        )
    }

    pub fn not_found(message: &str) -> JsonResponse<()> {
        JsonResponse(
            StatusCode::NOT_FOUND,
            Json(Response::failed(ErrCode::NotFound, Some(message))),
        )
    }

    pub fn internal_error(message: &str) -> JsonResponse<()> {
        JsonResponse(
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Response::failed(ErrCode::InternalServerError, Some(message))),
        )
    }
}

// ====================================
// 3. API 处理函数示例
// ====================================

/// 获取单个用户
pub async fn get_user(Path(user_id): Path<u64>) -> impl IntoResponse {
    // 模拟数据库查询
    if user_id == 0 {
        return JsonResponse::bad_request("用户ID不能为0");
    }

    if user_id == 999 {
        return JsonResponse::not_found("用户不存在");
    }

    let user = UserDto {
        id: user_id,
        username: format!("user_{}", user_id),
        email: format!("user_{}@example.com", user_id),
        active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    JsonResponse::success(user)
}

/// 获取用户列表（分页）
pub async fn list_users(Query(params): Query<PaginationQuery>) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let size = params.size.unwrap_or(10);

    // 参数验证
    if size > 100 {
        return JsonResponse::bad_request("每页大小不能超过100");
    }

    if page == 0 {
        return JsonResponse::bad_request("页码必须大于0");
    }

    // 模拟数据
    let users: Vec<UserDto> = (1..=size as u64)
        .map(|i| UserDto {
            id: (page - 1) as u64 * size as u64 + i,
            username: format!("user_{}", i),
            email: format!("user_{}@example.com", i),
            active: i % 2 == 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .collect();

    let total = 1000u64; // 模拟总数
    let pages = (total + size as u64 - 1) / size as u64;

    let response = PaginatedResponse {
        items: users,
        total,
        page,
        size,
        pages: pages as u32,
    };

    JsonResponse::success(response)
}

/// 创建用户
pub async fn create_user(Json(request): Json<CreateUserRequest>) -> impl IntoResponse {
    // 参数验证
    if request.username.is_empty() {
        return JsonResponse::bad_request("用户名不能为空");
    }

    if request.email.is_empty() {
        return JsonResponse::bad_request("邮箱不能为空");
    }

    if !request.email.contains('@') {
        return JsonResponse::bad_request("邮箱格式不正确");
    }

    if request.password.len() < 6 {
        return JsonResponse::bad_request("密码长度不能少于6位");
    }

    // 模拟创建用户
    let user = UserDto {
        id: 12345,
        username: request.username,
        email: request.email,
        active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    JsonResponse::created(user)
}

/// 更新用户
pub async fn update_user(
    Path(user_id): Path<u64>,
    Json(request): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    if user_id == 0 {
        return JsonResponse::bad_request("用户ID不能为0");
    }

    if user_id == 999 {
        return JsonResponse::not_found("用户不存在");
    }

    // 验证邮箱格式（如果提供）
    if let Some(ref email) = request.email {
        if !email.contains('@') {
            return JsonResponse::bad_request("邮箱格式不正确");
        }
    }

    // 模拟更新用户
    let user = UserDto {
        id: user_id,
        username: request.username.unwrap_or_else(|| format!("user_{}", user_id)),
        email: request.email.unwrap_or_else(|| format!("user_{}@example.com", user_id)),
        active: request.active.unwrap_or(true),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    JsonResponse::success(user)
}

/// 删除用户
pub async fn delete_user(Path(user_id): Path<u64>) -> impl IntoResponse {
    if user_id == 0 {
        return JsonResponse::bad_request("用户ID不能为0");
    }

    if user_id == 999 {
        return JsonResponse::not_found("用户不存在");
    }

    // 模拟删除操作
    JsonResponse::no_content()
}

// ====================================
// 4. 错误处理示例
// ====================================

/// 模拟数据库错误
pub async fn simulate_db_error() -> impl IntoResponse {
    JsonResponse::internal_error("数据库连接失败")
}

/// 模拟验证错误
pub async fn simulate_validation_error() -> impl IntoResponse {
    let mut errors = HashMap::new();
    errors.insert("username", vec!["用户名已存在"]);
    errors.insert("email", vec!["邮箱格式不正确", "邮箱已被使用"]);

    Json(Response::failed(
        ErrCode::ValidationError,
        Some("数据验证失败"),
    ))
    .set_data(errors)
    .into_response()
}

// ====================================
// 5. 复杂数据结构响应
// ====================================

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub user: UserDto,
    pub profile: ProfileDto,
    pub settings: UserSettings,
}

#[derive(Debug, Serialize)]
pub struct ProfileDto {
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserSettings {
    pub theme: String,
    pub language: String,
    pub notifications: NotificationSettings,
}

#[derive(Debug, Serialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub sms_notifications: bool,
}

/// 获取用户完整信息
pub async fn get_user_profile(Path(user_id): Path<u64>) -> impl IntoResponse {
    if user_id == 999 {
        return JsonResponse::not_found("用户不存在");
    }

    let profile = UserProfile {
        user: UserDto {
            id: user_id,
            username: format!("user_{}", user_id),
            email: format!("user_{}@example.com", user_id),
            active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        profile: ProfileDto {
            avatar: Some(format!("https://example.com/avatars/{}.jpg", user_id)),
            bio: Some("这是一个示例用户简介".to_string()),
            location: Some("北京, 中国".to_string()),
            website: Some("https://example.com".to_string()),
        },
        settings: UserSettings {
            theme: "dark".to_string(),
            language: "zh-CN".to_string(),
            notifications: NotificationSettings {
                email_notifications: true,
                push_notifications: false,
                sms_notifications: true,
            },
        },
    };

    JsonResponse::success(profile)
}

// ====================================
// 6. 文件上传响应
// ====================================

#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub file_id: String,
    pub filename: String,
    pub size: u64,
    pub content_type: String,
    pub url: String,
    pub uploaded_at: String,
}

pub async fn upload_file_response() -> impl IntoResponse {
    let response = FileUploadResponse {
        file_id: uuid::Uuid::new_v4().to_string(),
        filename: "example.jpg".to_string(),
        size: 1024 * 500, // 500KB
        content_type: "image/jpeg".to_string(),
        url: "https://cdn.example.com/files/example.jpg".to_string(),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    };

    JsonResponse::created(response)
}

// ====================================
// 7. 统计数据响应
// ====================================

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_users: u64,
    pub active_users: u64,
    pub total_orders: u64,
    pub revenue: f64,
    pub growth_rate: f64,
    pub recent_activities: Vec<ActivityDto>,
}

#[derive(Debug, Serialize)]
pub struct ActivityDto {
    pub id: u64,
    pub user_id: u64,
    pub action: String,
    pub timestamp: String,
    pub metadata: serde_json::Value,
}

pub async fn get_dashboard_stats() -> impl IntoResponse {
    let activities = vec![
        ActivityDto {
            id: 1,
            user_id: 123,
            action: "用户登录".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: serde_json::json!({"ip": "192.168.1.1", "device": "Chrome"}),
        },
        ActivityDto {
            id: 2,
            user_id: 456,
            action: "创建订单".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: serde_json::json!({"order_id": 789, "amount": 99.99}),
        },
    ];

    let stats = DashboardStats {
        total_users: 10000,
        active_users: 8500,
        total_orders: 15000,
        revenue: 1234567.89,
        growth_rate: 15.5,
        recent_activities: activities,
    };

    JsonResponse::success(stats)
}
