use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tonic::transport::Channel;
use tracing::{error, info};
use tracing_subscriber::filter::EnvFilter;

use shared::config::IdGeneratorRpcConfig;
use shared::proto::id_generator::{
    id_generator_service_client::IdGeneratorServiceClient, GenerateIdRequest,
};

pub fn new_id_generator_client(
    cfg: IdGeneratorRpcConfig,
) -> Result<IdGeneratorServiceClient<Channel>, Box<dyn std::error::Error>> {
    let endpoints = cfg.rpc_cfg.addr.into_iter().map(|a| {
        Channel::from_shared(a)
            .unwrap()
            .keep_alive_while_idle(true)
            .keep_alive_timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(5))
    });
    let channel = Channel::balance_list(endpoints);
    let client: IdGeneratorServiceClient<Channel> = IdGeneratorServiceClient::new(channel);
    Ok(client)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("Starting concurrent gRPC client test");

    // 创建共享的客户端实例
    let cfg = IdGeneratorRpcConfig::default();
    let client = Arc::new(tokio::sync::Mutex::new(new_id_generator_client(cfg)?));

    // 存储任务句柄
    let mut handles = Vec::new();

    // 创建并发任务
    for i in 0..10usize {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            info!("Starting REQUEST={}", i);

            // 使用超时包装请求
            let result = timeout(Duration::from_secs(5), async {
                let mut client_guard = client.lock().await;
                let req = tonic::Request::new(GenerateIdRequest {});
                client_guard.generate_id(req).await
            })
            .await;

            match result {
                Ok(Ok(resp)) => {
                    info!("SUCCESS REQUEST={}, RESPONSE={:?}", i, resp.into_inner());
                    Ok(())
                }
                Ok(Err(e)) => {
                    error!("GRPC_ERROR REQUEST={}, error={}", i, e);
                    Err(e)
                }
                Err(_) => {
                    error!("TIMEOUT REQUEST={}", i);
                    Err(tonic::Status::deadline_exceeded("Request timeout"))
                }
            }
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => {
                error_count += 1;
                error!("Task {} failed: {}", i, e);
            }
            Err(e) => {
                error_count += 1;
                error!("Task {} panicked: {}", i, e);
            }
        }
    }

    info!(
        "Completed: {} successful, {} errors",
        success_count, error_count
    );
    Ok(())
}
