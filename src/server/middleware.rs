use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName},
    middleware::Next,
    response::Response,
};
use opentelemetry::{
    propagation::{Extractor, Injector},
    trace::{SpanKind, Status, TraceContextExt, Tracer},
    KeyValue,
};
// 手动定义语义常量，因为版本兼容性问题
const HTTP_METHOD: &str = "http.method";
const HTTP_ROUTE: &str = "http.route";
const HTTP_STATUS_CODE: &str = "http.status_code";
const HTTP_URL: &str = "http.url";
const HTTP_USER_AGENT: &str = "http.user_agent";
use std::time::Instant;
use tracing::{error, info, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// HTTP Headers 作为 Extractor，用于从请求头中提取 trace context
struct HeaderExtractor<'a>(&'a HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|name| name.as_str()).collect()
    }
}

/// HTTP Headers 作为 Injector，用于向响应头中注入 trace context
struct HeaderInjector<'a>(&'a mut HeaderMap);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(header_name) = HeaderName::try_from(key) {
            if let Ok(header_value) = value.parse() {
                self.0.insert(header_name, header_value);
            }
        }
    }
}

/// Tracing 中间件配置
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// 是否记录请求体
    pub log_request_body: bool,
    /// 是否记录响应体
    pub log_response_body: bool,
    /// 慢请求阈值（毫秒）
    pub slow_request_threshold_ms: u64,
    /// 是否在响应头中包含 trace_id
    pub include_trace_id_header: bool,
    /// trace_id 响应头名称
    pub trace_id_header_name: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            log_request_body: false,
            log_response_body: false,
            slow_request_threshold_ms: 1000, // 1秒
            include_trace_id_header: true,
            trace_id_header_name: "x-trace-id".to_string(),
        }
    }
}

/// OpenTelemetry tracing 中间件
///
/// 该中间件会：
/// 1. 从请求头中提取 trace context
/// 2. 创建 HTTP span 并添加相关属性
/// 3. 记录请求开始和结束
/// 4. 处理错误和慢请求
/// 5. 在响应头中注入 trace context
pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    tracing_middleware_with_config(request, next, TracingConfig::default()).await
}

/// 带配置的 tracing 中间件
pub async fn tracing_middleware_with_config(
    request: Request,
    next: Next,
    config: TracingConfig,
) -> Response {
    let start_time = Instant::now();

    // 1. 提取请求信息
    let method = request.method().to_string().clone();
    let path = request.uri().path().to_string().clone();
    let query = request.uri().query().unwrap_or("");
    let uri = request.uri().to_string();
    let headers = request.headers().clone();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // 2. 从请求头中提取 trace context
    let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(&headers))
    });

    // 3. 创建新的 span
    let tracer = opentelemetry::global::tracer("tinyid");
    let span = tracer
        .span_builder(format!("{} {}", method, path))
        .with_kind(SpanKind::Server)
        .with_attributes([
            KeyValue::new(HTTP_METHOD, method.clone()),
            KeyValue::new(HTTP_URL, uri.clone()),
            KeyValue::new(HTTP_ROUTE, path.clone()),
            KeyValue::new(HTTP_USER_AGENT, user_agent.clone()),
        ])
        .start_with_context(&tracer, &parent_cx);

    let cx = parent_cx.with_span(span);

    // 调试：打印当前 span 的 trace_id
    let current_trace_id = cx.span().span_context().trace_id().to_string();
    info!("Current request trace_id: {}", current_trace_id);

    // 4. 创建 tracing span 并关联 OpenTelemetry context
    let tracing_span = tracing::info_span!(
        "http_request",
        method = %(method),
        path = %(path)    ,
        query = %query,
        user_agent = %user_agent,
    );
    tracing_span.set_parent(cx.clone());

    let _guard = tracing_span.enter();

    // 记录请求开始
    info!("Request started");

    // 5. 处理请求
    let mut response = next.run(request).await;

    // 6. 计算请求持续时间
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // 7. 获取响应状态码
    let status_code = response.status();
    let status_code_value = status_code.as_u16();

    // 8. 更新 OpenTelemetry span 属性
    let otel_ctx = tracing_span.context();
    otel_ctx
        .span()
        .set_attribute(KeyValue::new(HTTP_STATUS_CODE, status_code_value as i64));

    // 设置 span 状态
    if status_code.is_server_error() {
        otel_ctx
            .span()
            .set_status(Status::error("Internal server error"));
    } else if status_code.is_client_error() {
        otel_ctx.span().set_status(Status::error("Client error"));
    } else {
        otel_ctx.span().set_status(Status::Ok);
    }

    // 9. 记录日志
    match status_code_value {
        200..=299 => {
            if duration_ms >= config.slow_request_threshold_ms {
                warn!(
                    status_code = %status_code_value,
                    duration_ms = %duration_ms,
                    "Slow request completed"
                );
            } else {
                info!(
                    status_code = %status_code_value,
                    duration_ms = %duration_ms,
                    "Request completed successfully"
                );
            }
        }
        400..=499 => {
            warn!(
                status_code = %status_code_value,
                duration_ms = %duration_ms,
                "Client error occurred"
            );
        }
        500..=599 => {
            error!(
                status_code = %status_code_value,
                duration_ms = %duration_ms,
                "Server error occurred"
            );
        }
        _ => {
            info!(
                status_code = %status_code_value,
                duration_ms = %duration_ms,
                "Request completed with unknown status"
            );
        }
    }

    // 10. 在响应头中注入 trace context
    let response_headers = response.headers_mut();

    // 注入 trace context 到响应头
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(response_headers))
    });

    // 11. 添加 trace_id 到响应头（如果配置启用）
    if config.include_trace_id_header {
        let trace_id = otel_ctx.span().span_context().trace_id().to_string();
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::try_from(config.trace_id_header_name.as_str()),
            trace_id.parse(),
        ) {
            response_headers.insert(header_name, header_value);
        }
    }

    response
}

/// 错误处理中间件
///
/// 捕获并记录未处理的错误
pub async fn error_handling_middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    // 如果是服务器错误，记录详细信息
    if response.status().is_server_error() {
        error!(
            status_code = %response.status().as_u16(),
            "Unhandled server error occurred"
        );
    }

    response
}

/// 超时处理中间件的配置
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// 超时时间（秒）
    pub timeout_seconds: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30, // 30秒默认超时
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{body::Body, extract::Request, routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_tracing_middleware() {
        // 初始化测试用的 tracing
        crate::init_env();
        crate::metric::init_tracing();

        // 创建测试路由
        let app = Router::new()
            .route("/test", get(|| async { "test response" }))
            .layer(axum::middleware::from_fn(
                move |request: Request, next: Next| {
                    let config = TracingConfig::default();
                    Box::pin(
                        async move { tracing_middleware_with_config(request, next, config).await },
                    )
                },
            ));

        // 创建测试请求
        let request = Request::builder()
            .uri("/test")
            .header("user-agent", "test-agent")
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
