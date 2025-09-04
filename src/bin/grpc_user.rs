use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing::{error, info};

use tinyid::biz::HelloWorldUseCase;
use tinyid::config::ServerConfig;
use tinyid::data::UserDemoRepoImpl;
use tinyid::metric;
use tinyid::service::user_demo::user_demo_srv::user_demo_server::UserDemoServer;
use tinyid::service::user_demo::UserDemoSrvImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化环境变量
    tinyid::init_env();

    metric::init_tracing()?;

    let cfg = ServerConfig::new(
        String::from("0.0.0.0"),
        8080,
        vec!["[::1]:50052".to_string()],
    );

    let (server, cleanup) = init_app(cfg.clone())?;
    let (tx, mut rx) = mpsc::unbounded_channel();

    for addr in cfg.grpc_addr {
        let addr = addr.parse()?;
        let tx = tx.clone();
        let srv = Server::builder()
            .add_service(UserDemoServer::new(server.clone()))
            .serve(addr);
        tokio::spawn(async move {
            if let Err(e) = srv.await {
                error!("grpc server error: {}", e);
            }
            tx.send(()).unwrap();
        });
    }

    rx.recv().await;
    cleanup();
    Ok(())
}

fn init_app(cfg: ServerConfig) -> Result<(UserDemoSrvImpl, impl FnOnce())> {
    if cfg.grpc_addr.is_empty() {
        return Err(anyhow::anyhow!("grpc_addr is empty"));
    }
    // data
    let user_demo_repo = UserDemoRepoImpl::new();
    let user_demo_uc = Arc::new(UserDemoUseCase::new(Arc::new(user_demo_repo)));
    let service = UserDemoSrvImpl::new(user_demo_uc);

    let cleanup = || {
        info!("Cleaning up application resources");
    };

    Ok((service, cleanup))
}
