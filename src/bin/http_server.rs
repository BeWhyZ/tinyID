use anyhow::Result;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use std::sync::Arc;

use tinyid::biz::HelloWorldUseCase;
use tinyid::data::HelloWorldRepoImpl;
use tinyid::{config::ServerConfig, metric, server};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    metric::init_logs();

    // 加载配置

    // 启动metrics server

    // graceful shutdown
    let cancel_token = CancellationToken::new();
    let signal_cancel_token = cancel_token.clone();
    let shutdown_future = cancel_token.cancelled_owned();

    // start the shutdown signal
    tokio::spawn(async move {
        shutdown_signal().await;
        info!("Shutting down...");
        signal_cancel_token.cancel();
    });

    // 构建server
    let (app, cleanup) = init_app(ServerConfig::new(String::from("0.0.0.0"), 8080))?;

    // 启动server
    if let Err(e) = app.run_with_shutdown(shutdown_future).await {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    // 收到server退出信号

    // 清理资源
    cleanup();

    info!("Server shutdown complete");
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

fn init_app(cfg: ServerConfig) -> Result<(server::HttpServer, impl FnOnce())> {
    // data
    let hello_world_repo = HelloWorldRepoImpl::new(&cfg)?;
    let hello_world_uc = Arc::new(HelloWorldUseCase::new(Arc::new(hello_world_repo)));
    // TODO 优化这里的层级初始化问题。期望是每一个层级仅初始化一个上层即可，无需每次都来修改bin文件

    let server = server::HttpServer::new(Arc::new(cfg), hello_world_uc);
    let cleanup = || {
        info!("clean up resource");
    };
    Ok((server, cleanup))
}
