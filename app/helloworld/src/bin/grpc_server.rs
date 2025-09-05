use std::sync::Arc;

use anyhow::Result;
use shared::config::ServerConfig;
use shared::proto::id_generator::id_generator_service_server::IdGeneratorServiceServer;
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing::{error, info};

use tinyid::biz::{HelloWorldUseCase, UserDemoUseCase};
use tinyid::core::IDGenerator;
use tinyid::data::{new_user_client, HelloWorldRepoImpl};
use tinyid::service::HelloWorldService;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化环境变量
    shared::init_env();

    // 2. 初始化 tracing（统一入口）
    // very opinionated init of tracing, look at the source to make your own

    shared::init_tracing()?;

    let cfg = ServerConfig::new(
        String::from("0.0.0.0"),
        8080,
        vec!["[::1]:50051".to_string()],
    );

    let (server, cleanup) = init_app(cfg.clone())?;
    let (tx, mut rx) = mpsc::unbounded_channel();

    for addr in cfg.grpc_addr {
        let addr = addr.parse()?;
        let tx = tx.clone();
        let srv = Server::builder()
            .add_service(IdGeneratorServiceServer::new(server.clone()))
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

fn init_app(
    cfg: ServerConfig,
) -> Result<(
    HelloWorldService<HelloWorldRepoImpl, HelloWorldRepoImpl>,
    impl FnOnce(),
)> {
    if cfg.grpc_addr.is_empty() {
        return Err(anyhow::anyhow!("grpc_addr is empty"));
    }
    // data
    let id_generator = IDGenerator::new(cfg.id_generator.clone()).unwrap();
    let user_client = new_user_client(cfg.user_rpc.clone()).unwrap();
    let hello_world_repo = Arc::new(HelloWorldRepoImpl::new(
        Arc::new(id_generator),
        user_client,
    )?);
    let hello_world_uc = Arc::new(HelloWorldUseCase::new(hello_world_repo.clone()));
    let user_uc = Arc::new(UserDemoUseCase::new(hello_world_repo.clone()));
    let service = HelloWorldService::new(hello_world_uc, user_uc);

    let cleanup = || {
        info!("Cleaning up application resources");
    };

    Ok((service, cleanup))
}
