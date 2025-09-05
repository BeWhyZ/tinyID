use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub addr: String,
    pub port: u16,
    pub id_generator: IdGeneratorConfig,

    // grpc server 地址 [addr]:port, 可以有多个
    pub grpc_addr: Vec<String>,

    pub user_rpc: UserRpcConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub addr: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGeneratorRpcConfig {
    pub rpc_cfg: RpcConfig,
}

impl Default for IdGeneratorRpcConfig {
    fn default() -> Self {
        Self {
            rpc_cfg: RpcConfig {
                addr: vec!["http://[::1]:50051".to_string()],
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRpcConfig {
    pub rpc_cfg: RpcConfig,
}

impl Default for UserRpcConfig {
    fn default() -> Self {
        Self {
            rpc_cfg: RpcConfig {
                addr: vec!["http://[::1]:50052".to_string()],
            },
        }
    }
}

impl ServerConfig {
    pub fn new(addr: String, port: u16, grpc_addr: Vec<String>) -> Self {
        Self {
            addr,
            port,
            id_generator: IdGeneratorConfig::default(),
            grpc_addr,
            user_rpc: UserRpcConfig::default(),
        }
    }

    pub fn default_for_test() -> Self {
        Self {
            addr: "127.0.0.1".to_string(),
            port: 8080,
            id_generator: IdGeneratorConfig::default(),
            grpc_addr: vec!["[127.0.0.1]:50051".to_string()],
            user_rpc: UserRpcConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGeneratorConfig {
    /// 工作节点ID (0-1023)
    pub worker_id: u32,
    /// 数据中心ID (0-31)
    pub datacenter_id: u32,
    /// 序列号位数
    pub sequence_bits: u32,
    /// 工作节点ID位数
    pub worker_id_bits: u32,
    /// 数据中心ID位数
    pub datacenter_id_bits: u32,
    /// 时间戳位数
    pub timestamp_bits: u32,
    /// 起始时间戳 (毫秒)
    pub epoch: u64,
    /// 最大序列号
    pub max_sequence: u32,
    /// 最大工作节点ID
    pub max_worker_id: u32,
    /// 最大数据中心ID
    pub max_datacenter_id: u32,
}

impl Default for IdGeneratorConfig {
    fn default() -> Self {
        let sequence_bits = 12;
        let worker_id_bits: u32 = 7;
        let datacenter_id_bits = 3;
        let timestamp_bits = 41;

        Self {
            worker_id: 0,
            datacenter_id: 0,
            sequence_bits,
            worker_id_bits,
            datacenter_id_bits,
            timestamp_bits,
            epoch: 1735689600000, // 2025-01-01 00:00:00 UTC
            max_sequence: (1 << sequence_bits) - 1,
            max_worker_id: (1 << worker_id_bits) - 1,
            max_datacenter_id: (1 << datacenter_id_bits) - 1,
        }
    }
}
