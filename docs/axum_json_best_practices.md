# Axum JSON 响应最佳实践指南

## 目录
1. [基础概念](#基础概念)
2. [统一响应结构](#统一响应结构)
3. [错误处理策略](#错误处理策略)
4. [数据验证](#数据验证)
5. [性能优化](#性能优化)
6. [测试策略](#测试策略)
7. [实际应用案例](#实际应用案例)

## 基础概念

### 1. 使用 `axum::Json` 包装器

```rust
use axum::{response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

// ✅ 正确：使用 Json 包装器
async fn get_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
    })
}

// ❌ 错误：直接返回结构体
async fn get_user_wrong() -> User {
    User { id: 1, name: "Alice".to_string() }
}
```

### 2. 自动内容类型设置

Axum 的 `Json` 包装器会自动设置：
- `Content-Type: application/json`
- 正确的 JSON 序列化

## 统一响应结构

### 1. 设计原则

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T = ()>
where
    T: Serialize,
{
    /// 业务状态码
    pub code: ErrCode,
    /// 响应消息
    pub msg: String,
    /// 追踪引用（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// 响应数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}
```

### 2. 便利构造函数

```rust
impl<T> Response<T>
where
    T: Serialize,
{
    // 成功响应
    pub fn success(data: Option<T>) -> Self {
        // 实现...
    }

    // 失败响应
    pub fn failed(code: ErrCode, msg: Option<impl Into<String>>) -> Self {
        // 实现...
    }

    // 链式调用
    pub fn set_ref(mut self, r#ref: impl Into<String>) -> Self {
        self.r#ref = Some(r#ref.into());
        self
    }
}
```

### 3. 使用示例

```rust
// 成功响应
async fn create_user() -> Json<Response<User>> {
    let user = User { id: 1, name: "Alice".to_string() };
    Json(Response::success(Some(user)))
}

// 错误响应
async fn get_user_not_found() -> Json<Response<()>> {
    Json(Response::failed(
        ErrCode::NotFound,
        Some("用户不存在")
    ))
}

// 带追踪ID的响应
async fn create_user_with_ref() -> Json<Response<User>> {
    let user = User { id: 1, name: "Alice".to_string() };
    Json(Response::success(Some(user))
        .set_ref("req-12345"))
}
```

## 错误处理策略

### 1. 自定义错误类型

```rust
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("验证错误: {message}")]
    Validation { message: String },

    #[error("资源未找到: {resource}")]
    NotFound { resource: String },
}
```

### 2. 错误到响应的转换

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> AxumResponse {
        let (status_code, err_code, message) = match self {
            ApiError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrCode::DatabaseError,
                "数据库操作失败".to_string(),
            ),
            ApiError::Validation { message } => (
                StatusCode::BAD_REQUEST,
                ErrCode::ValidationError,
                message,
            ),
            // ... 其他错误映射
        };

        let response = Response::<()>::failed(err_code, Some(message));
        (status_code, Json(response)).into_response()
    }
}
```

### 3. Result 类型别名

```rust
pub type ApiResult<T> = Result<T, ApiError>;

// 使用示例
async fn get_user(id: u64) -> ApiResult<Json<Response<User>>> {
    let user = fetch_user_from_db(id).await?;
    Ok(Json(Response::success(Some(user))))
}
```

## 数据验证

### 1. 输入验证

```rust
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub struct Validator;

impl Validator {
    pub fn validate_email(email: &str) -> ApiResult<()> {
        if email.is_empty() {
            return Err(ApiError::validation("邮箱不能为空"));
        }
        if !email.contains('@') {
            return Err(ApiError::validation("邮箱格式不正确"));
        }
        Ok(())
    }
}
```

### 2. 批量验证

```rust
#[derive(Debug)]
pub struct ValidationErrors {
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        // 实现...
    }
}

impl IntoResponse for ValidationErrors {
    fn into_response(self) -> AxumResponse {
        let response = Response::failed(ErrCode::ValidationError, Some("数据验证失败"))
            .set_data(self.errors);
        (StatusCode::BAD_REQUEST, Json(response)).into_response()
    }
}
```

## 性能优化

### 1. 避免不必要的序列化

```rust
// ✅ 好：使用 skip_serializing_if
#[derive(Serialize)]
struct Response<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

// ❌ 不好：总是序列化 null 值
#[derive(Serialize)]
struct Response<T> {
    pub data: Option<T>,  // 会输出 "data": null
}
```

### 2. 流式响应（大数据）

```rust
use axum::response::Sse;
use futures::stream::{self, Stream};

async fn stream_large_data() -> impl IntoResponse {
    let stream = stream::iter(0..1000)
        .map(|i| {
            Ok::<_, Infallible>(Event::default()
                .json_data(json!({ "id": i, "data": format!("item_{}", i) }))
                .unwrap())
        });

    Sse::new(stream)
}
```

### 3. 条件序列化

```rust
#[derive(Serialize)]
struct User {
    pub id: u64,
    pub username: String,
    
    // 只在用户请求详细信息时序列化
    #[serde(skip_serializing_if = "should_skip_sensitive")]
    pub email: Option<String>,
}

fn should_skip_sensitive(email: &Option<String>) -> bool {
    // 根据上下文决定是否跳过敏感信息
    false
}
```

## 测试策略

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_create_user_success() {
        let app = create_app();
        let server = TestServer::new(app).unwrap();

        let response = server
            .post("/users")
            .json(&json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "password123"
            }))
            .await;

        response.assert_status_ok();
        
        let json: Response<User> = response.json();
        assert_eq!(json.code, ErrCode::Success);
        assert!(json.data.is_some());
    }

    #[tokio::test]
    async fn test_create_user_validation_error() {
        let app = create_app();
        let server = TestServer::new(app).unwrap();

        let response = server
            .post("/users")
            .json(&json!({
                "username": "",
                "email": "invalid-email",
                "password": "123"
            }))
            .await;

        response.assert_status(StatusCode::BAD_REQUEST);
        
        let json: Response<()> = response.json();
        assert_eq!(json.code, ErrCode::ValidationError);
    }
}
```

### 2. 集成测试

```rust
#[tokio::test]
async fn test_user_crud_workflow() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // 1. 创建用户
    let create_response = server
        .post("/users")
        .json(&json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }))
        .await;

    create_response.assert_status(StatusCode::CREATED);
    let user: Response<User> = create_response.json();
    let user_id = user.data.unwrap().id;

    // 2. 获取用户
    let get_response = server
        .get(&format!("/users/{}", user_id))
        .await;

    get_response.assert_status_ok();

    // 3. 更新用户
    let update_response = server
        .put(&format!("/users/{}", user_id))
        .json(&json!({
            "username": "updated_user"
        }))
        .await;

    update_response.assert_status_ok();

    // 4. 删除用户
    let delete_response = server
        .delete(&format!("/users/{}", user_id))
        .await;

    delete_response.assert_status(StatusCode::NO_CONTENT);
}
```

## 实际应用案例

### 1. 分页响应

```rust
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub size: u32,
    pub pages: u32,
}

async fn list_users(Query(params): Query<PaginationQuery>) -> ApiResult<Json<Response<PaginatedResponse<User>>>> {
    let (page, size) = Validator::validate_page_params(params.page, params.size)?;
    
    let users = fetch_users_from_db(page, size).await?;
    let total = count_users_from_db().await?;
    let pages = (total + size as u64 - 1) / size as u64;

    let response = PaginatedResponse {
        items: users,
        total,
        page,
        size,
        pages: pages as u32,
    };

    Ok(Json(Response::success(Some(response))))
}
```

### 2. 文件上传响应

```rust
#[derive(Serialize)]
pub struct FileUploadResponse {
    pub file_id: String,
    pub filename: String,
    pub size: u64,
    pub content_type: String,
    pub url: String,
    pub uploaded_at: String,
}

async fn upload_file(
    multipart: Multipart,
) -> ApiResult<Json<Response<FileUploadResponse>>> {
    // 处理文件上传...
    
    let response = FileUploadResponse {
        file_id: uuid::Uuid::new_v4().to_string(),
        filename: "uploaded_file.jpg".to_string(),
        size: 1024 * 500,
        content_type: "image/jpeg".to_string(),
        url: "https://cdn.example.com/files/uploaded_file.jpg".to_string(),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(Response::success(Some(response))))
}
```

### 3. 批量操作响应

```rust
#[derive(Serialize)]
pub struct BatchOperationResponse {
    pub total: u32,
    pub successful: u32,
    pub failed: u32,
    pub details: Vec<BatchOperationDetail>,
}

#[derive(Serialize)]
pub struct BatchOperationDetail {
    pub id: String,
    pub status: String,
    pub message: Option<String>,
}

async fn batch_delete_users(
    Json(request): Json<BatchDeleteRequest>,
) -> Json<Response<BatchOperationResponse>> {
    let mut details = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for user_id in request.user_ids {
        match delete_user_from_db(user_id).await {
            Ok(_) => {
                successful += 1;
                details.push(BatchOperationDetail {
                    id: user_id.to_string(),
                    status: "success".to_string(),
                    message: None,
                });
            }
            Err(e) => {
                failed += 1;
                details.push(BatchOperationDetail {
                    id: user_id.to_string(),
                    status: "failed".to_string(),
                    message: Some(e.to_string()),
                });
            }
        }
    }

    let response = BatchOperationResponse {
        total: successful + failed,
        successful,
        failed,
        details,
    };

    Json(Response::success(Some(response)))
}
```

## 安全考虑

### 1. 敏感信息过滤

```rust
#[derive(Serialize)]
pub struct PublicUser {
    pub id: u64,
    pub username: String,
    // 注意：不包含敏感信息如密码、内部ID等
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
        }
    }
}
```

### 2. 错误信息安全

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> AxumResponse {
        let (status_code, err_code, message) = match self {
            ApiError::Database(_) => {
                // ❌ 不要暴露数据库错误详情
                // error!("Database error: {}", e);
                
                // ✅ 记录详细错误到日志，返回通用消息
                tracing::error!("Database operation failed: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrCode::DatabaseError,
                    "服务暂时不可用".to_string(),
                )
            }
            // ... 其他错误处理
        };

        let response = Response::<()>::failed(err_code, Some(message));
        (status_code, Json(response)).into_response()
    }
}
```

## 总结

1. **统一结构**：使用一致的响应格式
2. **错误处理**：完善的错误类型和转换机制
3. **数据验证**：严格的输入验证和友好的错误提示
4. **性能优化**：合理的序列化策略
5. **安全考虑**：避免信息泄露
6. **可测试性**：编写完善的测试用例

这些最佳实践将帮助您构建健壮、高性能、安全的 Axum JSON API。
