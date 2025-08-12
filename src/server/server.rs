/*
 * @Author: iwhyz
 * @Date: 2025-07-24 10:00:00
 * @LastEditors: iwhyz
 * @LastEditTime: 2025-07-24 10:00:00
 * @Descriptiono
 * this server is used to how http server run
*/

use crate::{config::ServerConfig, Result};

pub struct HttpServer {
    pub addr: String,
    pub port: u16,
}

impl HttpServer {
    pub fn new(cfg: ServerConfig) -> Self {
        Self {
            addr: cfg.addr,
            port: cfg.port,
        }
    }

    pub async fn run(&self) -> Result<()> {
        Ok(())
    }

    pub async fn run_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()>,
    ) -> Result<()> {
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", self.addr, self.port)).await?;
        info!("Server is running on {}", listener.local_addr().unwrap());

        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| TinyidError::ServerError(e.to_string()))?;

        Ok(())
    }
}
