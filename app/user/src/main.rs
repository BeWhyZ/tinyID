mod biz;
mod data;
mod error;
mod service;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing::{error, info};

use shared::config::ServerConfig;
use shared::proto::user::user_demo_server::UserDemoServer;

use biz::UserUseCase;
use data::UserRepoImpl;
use service::UserDemoSrvImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化环境变量
    shared::init_env();

    shared::init_tracing()?;

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
    let user_demo_repo = UserRepoImpl::new();
    let user_demo_uc = Arc::new(UserUseCase::new(Arc::new(user_demo_repo)));
    let service = UserDemoSrvImpl::new(user_demo_uc);

    let cleanup = || {
        info!("Cleaning up application resources");
    };

    Ok((service, cleanup))
}
