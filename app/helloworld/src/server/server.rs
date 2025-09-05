/*
 * @Author: iwhyz
 * @Date: 2025-07-24 10:00:00
 * @LastEditors: iwhyz
 * @LastEditTime: 2025-07-24 10:00:00
 * @Descriptiono
 * this server is used to how http server run
*/
use std::sync::Arc;

use shared::{config::ServerConfig, metric};
use tracing::info;

use crate::biz::{HelloWorldUseCase, UserDemoUseCase};
use crate::data::HelloWorldRepoImpl;
use crate::{error::TinyIdError, service::HelloWorldServiceImpl, Result};

pub struct HttpServer {
    pub cfg: Arc<ServerConfig>,
    pub hello_world_service: Arc<HelloWorldServiceImpl>,
    pub metrics: Option<Arc<metric::AppMetrics>>,
}

impl HttpServer {
    pub fn new(
        cfg: Arc<ServerConfig>,
        huc: Arc<HelloWorldUseCase<HelloWorldRepoImpl>>,
        uuc: Arc<UserDemoUseCase<HelloWorldRepoImpl>>,
    ) -> Self {
        let hello_world_service = Arc::new(HelloWorldServiceImpl::new(huc, uuc));
        Self {
            cfg,
            hello_world_service,
            metrics: None,
        }
    }

    pub fn new_with_metrics(
        cfg: Arc<ServerConfig>,
        huc: Arc<HelloWorldUseCase<HelloWorldRepoImpl>>,
        uuc: Arc<UserDemoUseCase<HelloWorldRepoImpl>>,
        metrics: Arc<metric::AppMetrics>,
    ) -> Self {
        let hello_world_service = Arc::new(HelloWorldServiceImpl::new(huc, uuc));
        Self {
            cfg,
            hello_world_service,
            metrics: Some(metrics),
        }
    }

    pub async fn run(&self) -> Result<()> {
        Ok(())
    }

    pub async fn run_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", self.cfg.addr, self.cfg.port)).await?;
        info!("Server is running on {}", listener.local_addr().unwrap());

        let app = self.create_router();

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| TinyIdError::ServerError(e.to_string()))?;

        Ok(())
    }
}
