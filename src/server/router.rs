use axum::{response::Json, routing::get, Router};
use serde::Deserialize;
use std::sync::Arc;

use super::server::HttpServer;

impl HttpServer {
    pub fn create_router(&self) -> Router {
        let hello_service = Arc::clone(&self.hello_world_service);
        Router::new().route("/ping", get(|| async { "ok" })).route(
            "/id",
            get(move || {
                let service = Arc::clone(&hello_service);
                async move { Json(service.generate_id().await) }
            }),
        )
    }
}
