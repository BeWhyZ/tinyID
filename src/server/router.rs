use axum::{extract::Request, middleware::Next, response::Json, routing::get, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info_span, Span};

use super::{
    middleware::{tracing_middleware_with_config, TracingConfig},
    server::HttpServer,
};

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

    pub fn create_router_with_config(&self, tracing_config: TracingConfig) -> Router {
        let hello_service = Arc::clone(&self.hello_world_service);

        Router::new()
            // API 路由
            .route("/ping", get(|| async { "ok" }))
            .route("/health", get(self::health_check))
            .route(
                "/id",
                get(move || {
                    let service = Arc::clone(&hello_service);
                    async move { Json(service.generate_id().await) }
                }),
            )
            // 应用中间件层
            .layer(
                tower::ServiceBuilder::new()
                    // 请求超时
                    .layer(TimeoutLayer::new(Duration::from_secs(30)))
                    // 请求 ID 传播
                    .layer(PropagateRequestIdLayer::x_request_id())
                    .layer(SetRequestIdLayer::new(
                        axum::http::header::HeaderName::from_static("x-request-id"),
                        MyMakeRequestId::default(),
                    ))
                    // OpenTelemetry tracing
                    .layer(axum::middleware::from_fn(
                        move |request: Request, next: Next| {
                            let config = TracingConfig::default();
                            Box::pin(async move {
                                tracing_middleware_with_config(request, next, config).await
                            })
                        },
                    ))
                    // tower-http 的 TraceLayer 作为补充
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|request: &axum::http::Request<_>| {
                                let method = request.method();
                                let uri = request.uri();
                                let path = uri.path();
                                let query = uri.query().unwrap_or("");

                                info_span!(
                                    "tower_http",
                                    method = %method,
                                    path = %path,
                                    query = %query,
                                )
                            })
                            .on_request(|_request: &axum::http::Request<_>, _span: &Span| {
                                tracing::info!("Tower HTTP layer: Request started");
                            })
                            .on_response(
                                |response: &axum::http::Response<_>,
                                 latency: Duration,
                                 _span: &Span| {
                                    tracing::info!(
                                        status_code = %response.status(),
                                        latency_ms = %latency.as_millis(),
                                        "Tower HTTP layer: Response generated"
                                    );
                                },
                            )
                            .on_failure(
                                |error: tower_http::classify::ServerErrorsFailureClass,
                                 latency: Duration,
                                 _span: &Span| {
                                    tracing::error!(
                                        error = %error,
                                        latency_ms = %latency.as_millis(),
                                        "Tower HTTP layer: Request failed"
                                    );
                                },
                            ),
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
