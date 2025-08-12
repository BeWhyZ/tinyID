use core::panic::PanicInfo;

use tracing::error;

use tokio_util::sync::CancellationToken;

use tinyid::{config::ServerConfig, metric, server, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    metric::init_logs();

    // 加载配置

    // 启动metrics server

    // graceful shutdown
    let cancel_token = CancellationToken::new();
    let shutdown_token = cancel_token.clone();
    // start the shutdown signal
    tokio::spawn(async move {
        shutdown_signal().await;
        info!("Shutting down...");
        cancel_token.cancel();
    });

    // 构建server
    let server = server::HttpServer::new(ServerConfig::new(String::from("0.0.0.0"), 8080));

    // 启动server
    if let Err(e) = server.run_with_shutdown(shutdown_token.cancelled()).await {
        error!("Server error: {}", e);
        return Err(e);
    }

    // 收到server退出信号

    // 清理资源

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
