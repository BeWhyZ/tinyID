use axum::{extract::Request, middleware::Next, response::Json, routing::get, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use std::sync::Arc;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tracing::{info_span, Span};

use super::server::HttpServer;
use crate::service::HelloWorldServiceImpl;

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
        self.create_router_with_config()
    }

    pub fn create_router_with_config(&self) -> Router {
        let hello_service = Arc::<HelloWorldServiceImpl>::clone(&self.hello_world_service);

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
            // include trace context as header into the response
            .layer(OtelInResponseLayer::default())
            //start OpenTelemetry trace on incoming request
            .layer(OtelAxumLayer::default())
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
