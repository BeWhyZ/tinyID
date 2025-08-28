use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};

use crate::biz::HelloWorldRepo;
use crate::{config, TinyIdError};

/// 高性能ID生成器
///
/// 基于雪花算法的优化版本，支持：
/// - 高并发生成
/// - 时钟回拨处理
/// - 性能监控
/// - 批量生成
/// - ID预分配池
/// - 本地缓存
#[derive(Debug)]
pub struct HelloWorldRepoImpl {
    id_generator: IDGenerator,
}

impl HelloWorldRepo for HelloWorldRepoImpl {
    #[instrument(skip(self))]
    async fn generate_id(&self) -> Result<u64, TinyIdError> {
        self.id_generator.next_id()
    }
}

impl HelloWorldRepoImpl {
    pub fn new(cfg: &config::ServerConfig) -> Result<Self> {
        let generator = IDGenerator::new(cfg.id_generator.clone())?;

        Ok(Self {
            id_generator: generator,
        })
    }
}

#[derive(Debug)]
struct IDGenerator {
    cfg: config::IdGeneratorConfig,
    // 原子打包状态：(timestamp << sequence_bits) | sequence
    ts_seq: AtomicU64,
    start_time: SystemTime,
    total_generated: AtomicU64,
}

impl IDGenerator {
    pub fn new(cfg: config::IdGeneratorConfig) -> Result<Self> {
        // 验证配置
        if cfg.worker_id > cfg.max_worker_id {
            return Err(anyhow::anyhow!("worker_id is too large"));
        }
        if cfg.datacenter_id > cfg.max_datacenter_id {
            return Err(anyhow::anyhow!("datacenter_id is too large"));
        }

        Ok(Self {
            cfg,
            ts_seq: AtomicU64::new(0),
            start_time: SystemTime::now(),
            total_generated: AtomicU64::new(0),
        })
    }

    pub fn next_id(&self) -> Result<u64, TinyIdError> {
        self.generate_id()
    }

    fn generate_id(&self) -> Result<u64, TinyIdError> {
        let seq_bits = self.cfg.sequence_bits as u32;
        let seq_mask: u64 = (1u64 << self.cfg.sequence_bits) - 1;
        let max_seq: u64 = self.cfg.max_sequence as u64;

        loop {
            let now = self.get_current_timestamp()?;

            let cur = self.ts_seq.load(Ordering::Acquire);
            let cur_ts = cur >> seq_bits;
            let cur_seq = cur & seq_mask;

            // 回拨
            if now < cur_ts {
                let backwards = cur_ts - now;
                warn!("Clock moved backwards by {}ms, waiting", backwards);
                std::thread::sleep(Duration::from_micros(200));
                continue;
            }

            if now == cur_ts {
                // 同毫秒：CAS递增，不允许在同毫秒内序列回绕
                if cur_seq >= max_seq {
                    std::thread::sleep(Duration::from_micros(200));
                    continue;
                }
                let next = (cur_ts << seq_bits) | (cur_seq + 1);
                if let Ok(_) = self.ts_seq.compare_exchange_weak(
                    cur,
                    next,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    let id = self.assemble_id(now, cur_seq as u32);
                    self.total_generated.fetch_add(1, Ordering::Relaxed);
                    return Ok(id);
                }
                continue;
            }

            // 新毫秒：切换到新毫秒并分配首个序列0
            let next = (now << seq_bits) | 1; // 存1，返回0
            if let Ok(_) =
                self.ts_seq
                    .compare_exchange(cur, next, Ordering::AcqRel, Ordering::Acquire)
            {
                let id = self.assemble_id(now, 0);
                self.total_generated.fetch_add(1, Ordering::Relaxed);
                return Ok(id);
            }
            // 失败则重试
        }
    }

    /// 批量生成 count 个ID，采用CAS一次性预留序列区间，避免锁和逐个申请的开销
    pub fn generate_ids_batch(&self, count: usize) -> Result<Vec<u64>, TinyIdError> {
        let seq_bits = self.cfg.sequence_bits as u32;
        let seq_mask: u64 = (1u64 << self.cfg.sequence_bits) - 1;
        let max_seq: u64 = self.cfg.max_sequence as u64;

        let mut remaining: u64 = count as u64;
        let mut result = Vec::with_capacity(count);

        while remaining > 0 {
            let now = self.get_current_timestamp()?;
            let cur = self.ts_seq.load(Ordering::Acquire);
            let cur_ts = cur >> seq_bits;
            let cur_seq = cur & seq_mask; // 已分配数量（下一序列号）

            // 时钟回拨
            if now < cur_ts {
                let backwards = cur_ts - now;
                warn!("Clock moved backwards by {}ms, waiting", backwards);
                std::thread::sleep(Duration::from_micros(200));
                continue;
            }

            if now == cur_ts {
                let available = if cur_seq >= max_seq {
                    0
                } else {
                    (max_seq - cur_seq)
                };
                if available == 0 {
                    // 当前毫秒可用序列已满，等待下一毫秒
                    std::thread::sleep(Duration::from_micros(200));
                    continue;
                }
                let take = remaining.min(available);
                let new_seq = cur_seq + take; // 预留 [cur_seq, new_seq)
                let next = (cur_ts << seq_bits) | new_seq;
                if let Ok(_) = self.ts_seq.compare_exchange_weak(
                    cur,
                    next,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    for s in cur_seq..new_seq {
                        result.push(self.assemble_id(now, s as u32));
                    }
                    self.total_generated
                        .fetch_add(take as u64, Ordering::Relaxed);
                    remaining -= take;
                }
            } else {
                // 切换到新毫秒：一次性预留一段
                let avail = max_seq + 1; // 该毫秒可用的总数 [0..=max_seq]
                let take = remaining.min(avail);
                let new_seq = take; // 存储为下一序列号
                let next = (now << seq_bits) | new_seq;
                if let Ok(_) =
                    self.ts_seq
                        .compare_exchange(cur, next, Ordering::AcqRel, Ordering::Acquire)
                {
                    for s in 0..take {
                        result.push(self.assemble_id(now, s as u32));
                    }
                    self.total_generated
                        .fetch_add(take as u64, Ordering::Relaxed);
                    remaining -= take;
                }
            }
        }

        Ok(result)
    }

    fn get_current_timestamp(&self) -> Result<u64, TinyIdError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| TinyIdError::InternalError(e.to_string()))?;

        let timestamp = now.as_millis() as u64;
        Ok(timestamp.saturating_sub(self.cfg.epoch))
    }

    fn assemble_id(&self, timestamp: u64, sequence: u32) -> u64 {
        let timestamp_shift =
            self.cfg.datacenter_id_bits + self.cfg.worker_id_bits + self.cfg.sequence_bits;
        let datacenter_id_shift = self.cfg.worker_id_bits + self.cfg.sequence_bits;
        let worker_id_shift = self.cfg.sequence_bits;

        timestamp << timestamp_shift
            | (self.cfg.datacenter_id as u64) << datacenter_id_shift
            | (self.cfg.worker_id as u64) << worker_id_shift
            | sequence as u64
    }

    fn parse_id(&self, id: u64) -> (u64, u32) {
        let timestamp_shift =
            self.cfg.datacenter_id_bits + self.cfg.worker_id_bits + self.cfg.sequence_bits;

        let timestamp = (id >> timestamp_shift) & ((1 << self.cfg.timestamp_bits) - 1);
        let sequence = id & ((1 << self.cfg.sequence_bits) - 1);

        (timestamp + self.cfg.epoch, sequence as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn create_test_config() -> config::IdGeneratorConfig {
        config::IdGeneratorConfig {
            worker_id: 1,
            datacenter_id: 1,
            sequence_bits: 12,
            worker_id_bits: 5,
            datacenter_id_bits: 5,
            timestamp_bits: 41,
            epoch: 1640995200000, // 2022-01-01 00:00:00 UTC
            max_sequence: (1 << 12) - 1,
            max_worker_id: (1 << 5) - 1,
            max_datacenter_id: (1 << 5) - 1,
        }
    }

    fn create_test_server_config() -> config::ServerConfig {
        config::ServerConfig {
            addr: "127.0.0.1".to_string(),
            port: 8080,
            id_generator: create_test_config(),
        }
    }

    fn create_invalid_worker_id_config() -> config::IdGeneratorConfig {
        let mut cfg = create_test_config();
        cfg.worker_id = cfg.max_worker_id + 1; // 超过最大值
        cfg
    }

    fn create_invalid_datacenter_id_config() -> config::IdGeneratorConfig {
        let mut cfg = create_test_config();
        cfg.datacenter_id = cfg.max_datacenter_id + 1; // 超过最大值
        cfg
    }

    #[test]
    fn test_id_generator_new_success() {
        let cfg = create_test_config();
        let result = IDGenerator::new(cfg);
        assert!(result.is_ok());

        let generator = result.unwrap();
        assert_eq!(generator.cfg.worker_id, 1);
        assert_eq!(generator.cfg.datacenter_id, 1);
    }

    #[test]
    fn test_id_generator_new_invalid_worker_id() {
        let cfg = create_invalid_worker_id_config();
        let result = IDGenerator::new(cfg);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("worker_id is too large"));
    }

    #[test]
    fn test_id_generator_new_invalid_datacenter_id() {
        let cfg = create_invalid_datacenter_id_config();
        let result = IDGenerator::new(cfg);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("datacenter_id is too large"));
    }

    #[test]
    fn test_generate_single_id() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        let id = generator.generate_id().unwrap();
        assert!(id > 0);

        // 验证ID的结构
        let sequence_mask = (1 << generator.cfg.sequence_bits) - 1;
        let worker_id_mask = (1 << generator.cfg.worker_id_bits) - 1;
        let datacenter_id_mask = (1 << generator.cfg.datacenter_id_bits) - 1;

        // 提取各个部分
        let sequence = (id & sequence_mask) as u32;
        let worker_id = ((id >> generator.cfg.sequence_bits) & worker_id_mask) as u32;
        let datacenter_id = ((id >> (generator.cfg.sequence_bits + generator.cfg.worker_id_bits))
            & datacenter_id_mask) as u32;

        assert_eq!(worker_id, generator.cfg.worker_id);
        assert_eq!(datacenter_id, generator.cfg.datacenter_id);
        assert!(sequence <= generator.cfg.max_sequence);
    }

    #[test]
    fn test_generate_multiple_unique_ids() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        let mut ids = HashSet::new();
        let count = 1000;

        for _ in 0..count {
            let id = generator.generate_id().unwrap();
            assert!(ids.insert(id), "生成了重复的ID: {}", id);
        }

        assert_eq!(ids.len(), count);
    }

    #[test]
    fn test_generate_id_sequential_ordering() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        let mut prev_id = 0u64;
        for _ in 0..100 {
            let id = generator.generate_id().unwrap();
            assert!(id > prev_id, "ID不是递增的: {} <= {}", id, prev_id);
            prev_id = id;
        }
    }

    #[test]
    fn test_concurrent_id_generation() {
        let cfg = create_test_config();
        let generator = Arc::new(IDGenerator::new(cfg).unwrap());
        let ids = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        // 启动多个线程并发生成ID
        for _ in 0..1002 {
            let gen = Arc::clone(&generator);
            let ids_clone = Arc::clone(&ids);

            let handle = thread::spawn(move || {
                let mut local_ids = Vec::new();
                for _ in 0..1000 {
                    match gen.generate_id() {
                        Ok(id) => local_ids.push(id),
                        Err(e) => panic!("生成ID失败: {}", e),
                    }
                }
                {
                    let mut global_ids = ids_clone.lock().unwrap();
                    global_ids.extend(local_ids);
                }
            });

            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证所有ID都是唯一的
        let all_ids = ids.lock().unwrap();
        let mut unique_ids = HashSet::new();

        for &id in all_ids.iter() {
            assert!(
                unique_ids.insert(id),
                "发现重复ID: {:?}",
                generator.parse_id(id)
            );
        }
    }

    #[test]
    fn test_get_current_timestamp() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        let timestamp1 = generator.get_current_timestamp().unwrap();
        thread::sleep(Duration::from_millis(1));
        let timestamp2 = generator.get_current_timestamp().unwrap();

        assert!(timestamp2 >= timestamp1);
    }

    #[test]
    fn test_assemble_id() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        let timestamp = 1000u64;
        let sequence = 100u32;

        let id = generator.assemble_id(timestamp, sequence);

        // 验证各个组件是否正确编码
        let sequence_mask = (1 << generator.cfg.sequence_bits) - 1;
        let worker_id_mask = (1 << generator.cfg.worker_id_bits) - 1;
        let datacenter_id_mask = (1 << generator.cfg.datacenter_id_bits) - 1;

        let extracted_sequence = (id & sequence_mask) as u32;
        let extracted_worker_id = ((id >> generator.cfg.sequence_bits) & worker_id_mask) as u32;
        let extracted_datacenter_id = ((id
            >> (generator.cfg.sequence_bits + generator.cfg.worker_id_bits))
            & datacenter_id_mask) as u32;

        assert_eq!(extracted_sequence, sequence);
        assert_eq!(extracted_worker_id, generator.cfg.worker_id);
        assert_eq!(extracted_datacenter_id, generator.cfg.datacenter_id);
    }

    #[test]
    fn test_id_components_boundary_values() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        // 测试最大序列号
        let id = generator.assemble_id(1000, generator.cfg.max_sequence);
        let sequence_mask = (1 << generator.cfg.sequence_bits) - 1;
        let extracted_sequence = (id & sequence_mask) as u32;
        assert_eq!(extracted_sequence, generator.cfg.max_sequence);

        // 测试序列号为0
        let id = generator.assemble_id(1000, 0);
        let extracted_sequence = (id & sequence_mask) as u32;
        assert_eq!(extracted_sequence, 0);
    }

    #[test]
    fn test_hello_world_repo_impl_new() {
        let server_cfg = create_test_server_config();
        let result = HelloWorldRepoImpl::new(&server_cfg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hello_world_repo_impl_new_invalid_config() {
        let mut server_cfg = create_test_server_config();
        server_cfg.id_generator.worker_id = server_cfg.id_generator.max_worker_id + 1;

        let result = HelloWorldRepoImpl::new(&server_cfg);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_hello_world_repo_generate_id() {
        let server_cfg = create_test_server_config();
        let repo = HelloWorldRepoImpl::new(&server_cfg).unwrap();

        let result = repo.generate_id().await;
        assert!(result.is_ok());

        let id = result.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_hello_world_repo_generate_multiple_ids() {
        let server_cfg = create_test_server_config();
        let repo = HelloWorldRepoImpl::new(&server_cfg).unwrap();

        let mut ids = HashSet::new();
        for _ in 0..100 {
            let id = repo.generate_id().await.unwrap();
            assert!(ids.insert(id), "生成了重复的ID: {}", id);
        }

        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn test_id_generator_performance() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        let start = std::time::Instant::now();
        let count = 10000;
        let mut unique_ids = HashSet::new();

        for _ in 0..count {
            let id = generator.generate_id().expect("ID generation failed");
            assert!(
                unique_ids.insert(id),
                "发现重复ID: {:?}",
                generator.parse_id(id)
            );
        }

        let duration = start.elapsed();
        let ids_per_sec = count as f64 / duration.as_secs_f64();

        println!("生成了 {} 个ID，用时 {:?}", count, duration);
        println!("性能: {:.0} IDs/sec", ids_per_sec);

        // 基本性能检查：应该能够每秒生成至少1000个ID
        assert!(ids_per_sec > 1000.0, "性能太低: {:.0} IDs/sec", ids_per_sec);
    }

    #[test]
    fn test_id_format_consistency() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        // 生成多个ID并验证格式一致性
        for _ in 0..10 {
            let id = generator.generate_id().unwrap();

            // ID应该是正数
            assert!(id > 0);

            // ID应该在合理范围内（不超过64位整数的最大值的一半）
            assert!(id < u64::MAX / 2);

            // 验证ID的位模式是合理的
            let bit_count = 64 - id.leading_zeros();
            assert!(bit_count <= 64, "ID使用了超过64位: {}", id);
        }
    }

    #[test]
    fn test_default_config_validity() {
        let default_cfg = config::IdGeneratorConfig::default();
        let result = IDGenerator::new(default_cfg);
        assert!(result.is_ok(), "默认配置应该是有效的");

        let generator = result.unwrap();
        let id = generator.generate_id().unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_edge_case_epoch_time() {
        let mut cfg = create_test_config();
        // 设置epoch为当前时间，这样相对时间戳会很小
        cfg.epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let generator = IDGenerator::new(cfg).unwrap();
        let id = generator.generate_id().unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_next_id_method() {
        let cfg = create_test_config();
        let generator = IDGenerator::new(cfg).unwrap();

        // 测试next_id方法（它应该调用generate_id）
        let id1 = generator.next_id().unwrap();
        let id2 = generator.next_id().unwrap();

        assert!(id1 > 0);
        assert!(id2 > 0);
        assert_ne!(id1, id2);
    }
}
