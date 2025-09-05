use std::sync::Arc;

use anyhow::{Context, Result};
use shared::{config::ServerConfig, metric};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use tinyid::biz::{HelloWorldUseCase, UserDemoUseCase};
use tinyid::core::IDGenerator;
use tinyid::data::{new_user_client, HelloWorldRepoImpl};
use tinyid::server;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化环境变量
    shared::init_env();

    // 2. 初始化 tracing（统一入口）
    // very opinionated init of tracing, look at the source to make your own

    shared::init_tracing()?;

    info!("TinyID HTTP Server starting...");

    // 3. 初始化 metrics 系统
    let (metrics_server, app_metrics) = metric::init_metrics()
        .map_err(|e| anyhow::anyhow!("Failed to initialize metrics: {}", e))?;

    // 4. 设置优雅关闭
    let cancel_token = CancellationToken::new();
    let signal_cancel_token = cancel_token.clone();
    let metrics_cancel_token = cancel_token.clone();
    let shutdown_future = cancel_token.cancelled_owned();

    // 5. 启动关闭信号监听
    tokio::spawn(async move {
        shutdown_signal().await;
        info!("Shutdown signal received, initiating graceful shutdown...");
        signal_cancel_token.cancel();
    });

    // 6. 启动 metrics 服务器
    let metrics_handle = {
        let metrics_shutdown = metrics_cancel_token.cancelled_owned();
        tokio::spawn(async move {
            if let Err(e) = metrics_server.start_with_shutdown(metrics_shutdown).await {
                error!("Metrics server error: {}", e);
            }
        })
    };

    // 7. 构建主应用服务器
    let (app, cleanup) = init_app(
        ServerConfig::new(String::from("0.0.0.0"), 8080, vec![]),
        app_metrics,
    )?;

    // 8. 启动主服务器
    info!("Starting main HTTP server...");
    let server_result = app.run_with_shutdown(shutdown_future).await;

    // 9. 等待 metrics 服务器关闭
    info!("Waiting for metrics server to shutdown...");
    if let Err(e) = metrics_handle.await {
        warn!("Metrics server task error: {}", e);
    }

    // 10. 清理资源
    info!("Cleaning up resources...");
    cleanup();

    // 11. 检查服务器错误
    if let Err(e) = server_result {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    info!("TinyID HTTP Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => info!("Received SIGTERM, shutting down..."),
            _ = sigint.recv() => info!("Received SIGINT, shutting down..."),
            _ = tokio::signal::ctrl_c() => info!("Received CTRL+C, shutting down..."),
        }
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
        info!("Received CTRL+C, shutting down...");
    }
}

fn init_app(
    cfg: ServerConfig,
    app_metrics: Arc<metric::AppMetrics>,
) -> Result<(server::HttpServer, impl FnOnce())> {
    // data
    let id_generator = IDGenerator::new(cfg.id_generator.clone()).unwrap();
    let user_client = new_user_client(cfg.user_rpc.clone()).unwrap();
    let hello_world_repo = Arc::new(HelloWorldRepoImpl::new(
        Arc::new(id_generator),
        user_client,
    )?);
    let hello_world_uc = Arc::new(HelloWorldUseCase::new(hello_world_repo.clone()));
    let user_uc = Arc::new(UserDemoUseCase::new(hello_world_repo.clone()));
    // TODO 优化这里的层级初始化问题。期望是每一个层级仅初始化一个上层即可，无需每次都来修改bin文件

    let server = server::HttpServer::new_with_metrics(
        Arc::new(cfg.clone()),
        hello_world_uc,
        user_uc,
        app_metrics,
    );

    let cleanup = || {
        info!("Cleaning up application resources");
    };

    Ok((server, cleanup))
}
