use axum::{routing::get, Router};

impl HttpServer {
    pub fn create_router(&self) -> Router {
        Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/health", get(|| async { "OK" }))
            .route("/metrics", get(|| async { "Metrics" }))
            .route("/shutdown", post(|| async { "Shutting down..." }))
            .route("/panic", get(|| async { panic!("Panic") }))
            .route("/error", get(|| async { Err("Error") }))
            .route(
                "/timeout",
                get(|| async {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    "Timeout"
                }),
            )
            .route("/redirect", get(|| async { Redirect::temporary("/") }))
            .route(
                "/json",
                get(|| async { Json(json!({ "message": "Hello, World!" })) }),
            )
    }
}
