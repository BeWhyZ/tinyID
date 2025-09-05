use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{error, info, warn, Span};
use uuid::Uuid;

/// 最佳实践的 Tracing 中间件配置
#[derive(Debug, Clone)]
pub struct OptimalTracingConfig {
    /// 慢请求阈值（毫秒）
    pub slow_request_threshold_ms: u64,
    /// 是否在响应头中包含 trace_id
    pub include_trace_id_header: bool,
    /// trace_id 响应头名称
    pub trace_id_header_name: String,
    /// 是否记录请求详情
    pub log_request_details: bool,
}

impl Default for OptimalTracingConfig {
    fn default() -> Self {
        Self {
            slow_request_threshold_ms: 1000,
            include_trace_id_header: true,
            trace_id_header_name: "x-trace-id".to_string(),
            log_request_details: true,
        }
    }
}

/// 最佳实践的 OpenTelemetry tracing 中间件
///
/// 设计原则：
/// 1. 使用纯 tracing crate，通过 OpenTelemetryLayer 自动导出
/// 2. 每个请求独立的 span，避免span聚合
/// 3. 遵循 OpenTelemetry 语义约定
/// 4. 最小化性能开销
pub async fn optimal_tracing_middleware(request: Request, next: Next) -> Response {
    optimal_tracing_middleware_with_config(request, next, OptimalTracingConfig::default()).await
}

/// 带配置的最佳实践 tracing 中间件
pub async fn optimal_tracing_middleware_with_config(
    request: Request,
    next: Next,
    config: OptimalTracingConfig,
) -> Response {
    let start_time = Instant::now();

    // 1. 提取请求信息
    let method = request.method();
    let uri = request.uri();
    let path = uri.path();
    let query = uri.query().unwrap_or("");
    let headers = request.headers();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // 2. 为每个请求生成唯一的请求ID
    let request_id = Uuid::new_v4();

    // 3. 创建独立的 tracing span（推荐方式）
    // OpenTelemetryLayer 会自动：
    // - 提取父级 trace context
    // - 创建 OpenTelemetry span
    // - 设置正确的 span 属性
    // - 导出到配置的后端
    let span = tracing::info_span!(
        "http_request",
        // OpenTelemetry 语义约定字段
        "http.method" = %method,
        "http.route" = %path,
        "http.url" = %uri,
        "http.user_agent" = %user_agent,
        "http.request_content_length" = tracing::field::Empty,
        "http.response.status_code" = tracing::field::Empty,
        "http.response_content_length" = tracing::field::Empty,

        // 自定义字段
        "request.id" = %request_id,
        "request.query" = %query,

        // 用于过滤的标签
        "service.name" = "tinyid",
        "span.kind" = "server",
    );

    // 4. 进入 span 上下文
    let _guard = span.enter();

    // 5. 记录请求开始（结构化日志）
    if config.log_request_details {
        info!(
            request.method = %method,
            request.path = %path,
            request.query = %query,
            request.user_agent = %user_agent,
            request.id = %request_id,
            "HTTP request started"
        );
    }

    // 6. 处理请求
    let mut response = next.run(request).await;

    // 7. 计算请求时长
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // 8. 获取响应信息
    let status_code = response.status();
    let status_code_value = status_code.as_u16();

    // 9. 更新 span 字段（运行时设置）
    Span::current().record("http.response.status_code", status_code_value);

    // 10. 根据响应状态记录结构化日志
    match status_code_value {
        200..=299 => {
            if duration_ms >= config.slow_request_threshold_ms {
                warn!(
                    request.id = %request_id,
                    response.status_code = %status_code_value,
                    response.duration_ms = %duration_ms,
                    "Slow HTTP request completed"
                );
            } else {
                info!(
                    request.id = %request_id,
                    response.status_code = %status_code_value,
                    response.duration_ms = %duration_ms,
                    "HTTP request completed successfully"
                );
            }
        }
        400..=499 => {
            warn!(
                request.id = %request_id,
                response.status_code = %status_code_value,
                response.duration_ms = %duration_ms,
                "HTTP request failed with client error"
            );
        }
        500..=599 => {
            error!(
                request.id = %request_id,
                response.status_code = %status_code_value,
                response.duration_ms = %duration_ms,
                "HTTP request failed with server error"
            );
        }
        _ => {
            info!(
                request.id = %request_id,
                response.status_code = %status_code_value,
                response.duration_ms = %duration_ms,
                "HTTP request completed with unknown status"
            );
        }
    }

    // 11. 添加 trace_id 到响应头（如果配置启用）
    if config.include_trace_id_header {
        // 获取当前 span 的 trace_id
        let current_span = Span::current();
        if let Some(context) = current_span.context().span().span_context() {
            if context.is_valid() {
                let trace_id = context.trace_id().to_string();
                if let (Ok(header_name), Ok(header_value)) = (
                    HeaderName::try_from(config.trace_id_header_name.as_str()),
                    trace_id.parse(),
                ) {
                    response.headers_mut().insert(header_name, header_value);
                }
            }
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{body::Body, extract::Request, routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_optimal_tracing_middleware() {
        // 初始化测试用的 tracing
        shared::init_env();
        let _cleanup = shared::init_tracing().expect("Failed to init tracing");

        // 创建测试路由
        let app = Router::new()
            .route("/test", get(|| async { "test response" }))
            .layer(axum::middleware::from_fn(optimal_tracing_middleware));

        // 创建测试请求
        let request = Request::builder()
            .uri("/test?id=123")
            .header("user-agent", "test-agent/1.0")
            .body(Body::empty())
            .unwrap();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应
        assert_eq!(response.status(), StatusCode::OK);

        // 验证响应头中包含 trace_id
        assert!(response.headers().contains_key("x-trace-id"));
    }
}
