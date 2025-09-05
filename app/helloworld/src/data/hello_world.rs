use std::sync::Arc;

use anyhow::Result;
use shared::proto::user::{user_demo_client::UserDemoClient, GetUserRequest, User};
use tonic::{transport::Channel, Request};
use tracing::{error, instrument};

use crate::biz::{HelloWorldRepo, UserDemoRepo};
use crate::core::IDGenerator;
use crate::TinyIdError;

/// 高性能ID生成器
///
/// 基于雪花算法的优化版本，支持：
/// - 高并发生成
/// - 时钟回拨处理
/// - 性能监控
/// - 批量生成
/// - ID预分配池
/// - 本地缓存
#[derive(Debug, Clone)]
pub struct HelloWorldRepoImpl {
    ig: Arc<IDGenerator>,
    user_client: UserDemoClient<Channel>,
}

impl HelloWorldRepo for HelloWorldRepoImpl {
    #[instrument(skip(self))]
    async fn generate_id(&self) -> Result<u64, TinyIdError> {
        self.ig.next_id()
    }
}

impl UserDemoRepo for HelloWorldRepoImpl {
    #[instrument(skip(self))]
    async fn get_user(&self, id: u64) -> Result<User, TinyIdError> {
        let resp = self
            .user_client
            .clone()
            .get_user(Request::new(GetUserRequest { id }))
            .await;
        match resp {
            Ok(resp) => resp
                .into_inner()
                .user
                .ok_or(TinyIdError::UserServiceError("user not found".to_string())),
            Err(e) => {
                error!("get user failed: {}", e);
                Err(TinyIdError::UserServiceError(e.to_string()))
            }
        }
    }
}

impl<'a> HelloWorldRepoImpl {
    pub fn new(generator: Arc<IDGenerator>, user_client: UserDemoClient<Channel>) -> Result<Self> {
        Ok(Self {
            ig: generator,
            user_client,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use shared::config::{IdGeneratorConfig, ServerConfig};
//     use std::collections::HashSet;
//     use std::sync::Arc;
//     use super::rpc::new_user_client;

//     fn create_test_config() -> IdGeneratorConfig {
//         IdGeneratorConfig {
//             worker_id: 1,
//             datacenter_id: 1,
//             sequence_bits: 12,
//             worker_id_bits: 5,
//             datacenter_id_bits: 5,
//             timestamp_bits: 41,
//             epoch: 1640995200000, // 2022-01-01 00:00:00 UTC
//             max_sequence: (1 << 12) - 1,
//             max_worker_id: (1 << 5) - 1,
//             max_datacenter_id: (1 << 5) - 1,
//         }
//     }

//     fn create_test_server_config() -> ServerConfig {
//         ServerConfig::default_for_test()
//     }

//     fn create_invalid_worker_id_config() -> IdGeneratorConfig {
//         let mut cfg = create_test_config();
//         cfg.worker_id = cfg.max_worker_id + 1; // 超过最大值
//         cfg
//     }

//     fn create_invalid_datacenter_id_config() -> IdGeneratorConfig {
//         let mut cfg = create_test_config();
//         cfg.datacenter_id = cfg.max_datacenter_id + 1; // 超过最大值
//         cfg
//     }

//     #[test]
//     fn test_hello_world_repo_impl_new() {
//         let server_cfg = create_test_server_config();
//         let id_generator = IDGenerator::new(server_cfg.id_generator).unwrap();
//         let result = HelloWorldRepoImpl::new(Arc::new(id_generator));
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_hello_world_repo_impl_new_invalid_config() {
//         let mut server_cfg = create_test_server_config();
//         server_cfg.id_generator.worker_id = server_cfg.id_generator.max_worker_id + 1;
//         let id_generator = IDGenerator::new(server_cfg.id_generator).unwrap();
//         let result = HelloWorldRepoImpl::new(Arc::new(id_generator));
//         assert!(result.is_err());
//     }

//     #[tokio::test]
//     async fn test_hello_world_repo_generate_id() {
//         let server_cfg = create_test_server_config();
//         let id_generator = IDGenerator::new(server_cfg.id_generator).unwrap();
//         let repo = HelloWorldRepoImpl::new(Arc::new(id_generator)).unwrap();

//         let result = repo.generate_id().await;
//         assert!(result.is_ok());

//         let id = result.unwrap();
//         assert!(id > 0);
//     }

//     #[tokio::test]
//     async fn test_hello_world_repo_generate_multiple_ids() {
//         let server_cfg = create_test_server_config();
//         let id_generator = IDGenerator::new(server_cfg.id_generator).unwrap();
//         let repo = HelloWorldRepoImpl::new(Arc::new(id_generator)).unwrap();

//         let mut ids = HashSet::new();
//         for _ in 0..100 {
//             let id = repo.generate_id().await.unwrap();
//             assert!(ids.insert(id), "生成了重复的ID: {}", id);
//         }

//         assert_eq!(ids.len(), 100);
//     }
// }
