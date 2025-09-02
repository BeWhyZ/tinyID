use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing::{error, info};

use tinyid::biz::HelloWorldUseCase;
use tinyid::config::ServerConfig;
use tinyid::data::{HelloWorldRepoImpl, IDGenerator};
use tinyid::service::id_generator::id_generator_service_server::IdGeneratorServiceServer;
use tinyid::service::HelloWorldService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

fn init_app(cfg: ServerConfig) -> Result<(HelloWorldService<HelloWorldRepoImpl>, impl FnOnce())> {
    if cfg.grpc_addr.is_empty() {
        return Err(anyhow::anyhow!("grpc_addr is empty"));
    }
    // data
    let id_generator = IDGenerator::new(cfg.id_generator.clone()).unwrap();
    let hello_world_repo = HelloWorldRepoImpl::new(Arc::new(id_generator))?;
    let hello_world_uc = Arc::new(HelloWorldUseCase::new(Arc::new(hello_world_repo)));
    let service = HelloWorldService::new(hello_world_uc);

    let cleanup = || {
        info!("Cleaning up application resources");
    };

    Ok((service, cleanup))
}
