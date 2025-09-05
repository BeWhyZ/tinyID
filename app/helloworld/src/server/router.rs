use std::sync::Arc;
use std::time::Duration;

use axum::{response::Json, routing::get, Router};
use tower_http::{
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info_span, Span};

use super::{middleware::TracingConfig, server::HttpServer};

/// 自定义请求 ID 生成器
#[derive(Clone, Default)]
struct MyMakeRequestId;

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let request_id = uuid::Uuid::new_v4().to_string();
        Some(RequestId::new(request_id.parse().ok()?))
    }
}

impl HttpServer {
    pub fn create_router(&self) -> Router {
        self.create_router_with_config(TracingConfig::default())
    }

    pub fn create_router_with_config(&self, _tracing_config: TracingConfig) -> Router {
        let hello_service = Arc::clone(&self.hello_world_service);

        Router::new()
            // API 路由
            .route("/ping", get(|| async { "ok" }))
            .route("/health", get(self::health_check))
            .route(
                "/id",
                get({
                    let service = hello_service.clone();
                    move || async move { service.generate_id().await }
                }),
            )
            .route(
                "/user",
                get({
                    let service = hello_service.clone();
                    move |query| async move { service.get_user(query).await }
                }),
            )
            // 应用中间件层
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MyMakeRequestId::default()))
            // 使用简化的 TraceLayer，让 OpenTelemetryLayer 自动处理
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &axum::http::Request<_>| {
                        let method = request.method();
                        let uri = request.uri();
                        let path = uri.path();
                        let query = uri.query().unwrap_or("");
                        let request_id = uuid::Uuid::new_v4();

                        info_span!(
                            "http_request",
                            // 使用OpenTelemetry语义约定
                            "http.method" = %method,
                            "http.route" = %path,
                            "http.url" = %uri,
                            "request.id" = %request_id,
                            "request.query" = %query,
                        )
                    })
                    .on_request(|_request: &axum::http::Request<_>, _span: &Span| {
                        tracing::info!("Processing HTTP request");
                    })
                    .on_response(
                        |response: &axum::http::Response<_>, latency: Duration, _span: &Span| {
                            tracing::info!(
                                "http.response.status_code" = %response.status(),
                                duration_ms = %latency.as_millis(),
                                "HTTP request completed"
                            );
                        },
                    )
                    .on_failure(
                        |error: tower_http::classify::ServerErrorsFailureClass,
                         latency: Duration,
                         _span: &Span| {
                            tracing::error!(
                                error = %error,
                                duration_ms = %latency.as_millis(),
                                "HTTP request failed"
                            );
                        },
                    ),
            )
    }
}

/// 健康检查端点
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "tinyid",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
