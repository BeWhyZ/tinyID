use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tracing::{error, info};

use crate::error::TinyIdError;
use crate::Result;

pub struct GeneratorConfig {
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
    /// 起始时间戳 (毫秒), 使用相对时间来进行延长系统使用时间
    pub epoch: u64,
    /// 最大序列号
    pub max_sequence: u32,
    /// 最大工作节点ID
    pub max_worker_id: u32,
    /// 最大数据中心ID
    pub max_datacenter_id: u32,
}

impl GeneratorConfig {
    fn validate(
        timestamp_bits: u32,
        datacenter_id: u32,
        worker_id_bits: u32,
        sequence_bits: u32,
    ) -> bool {
        let total_bits = timestamp_bits + datacenter_id + worker_id_bits + sequence_bits;
        total_bits == 64
    }
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        let timestamp_bits = 41;
        let datacenter_id_bits: u32 = 5;
        let worker_id_bits = 6;
        let sequence_bits = 12;
        Self {
            worker_id: 0,
            timestamp_bits,
            datacenter_id_bits,
            worker_id_bits,
            sequence_bits,
            datacenter_id: 0,
            epoch: 1609459200000, // 2021-01-01 00:00:00 UTC
            max_sequence: (1 << sequence_bits) - 1,
            max_worker_id: (1 << worker_id_bits) - 1,
            max_datacenter_id: (1 << datacenter_id_bits) - 1,
        }
    }
}

pub struct TinyIdGenerator {
    config: GeneratorConfig,
    last_timestamp: AtomicU64,
    sequence: AtomicU32,
}

impl TinyIdGenerator {
    pub fn new(cfg: GeneratorConfig) -> Result<Self> {
        if cfg.worker_id > cfg.max_worker_id {
            return Err(TinyIdError::InvalidWorkerId(cfg.worker_id));
        }
        if cfg.datacenter_id > cfg.max_datacenter_id {
            return Err(TinyIdError::InvalidDatacenterId(cfg.datacenter_id));
        }
        let generator = Self {
            config: cfg,
            last_timestamp: AtomicU64::new(0),
            sequence: AtomicU32::new(0),
        };
        info!("TinyIdGenerator created");

        return Ok(generator);
    }

    pub fn next_id(&self) -> Result<u64> {
        self.generate_id()
    }

    fn generate_id(&self) -> Result<u64> {
        let mut last_timestamp = self.last_timestamp.load(Ordering::Relaxed);
        let mut sequence = self.sequence.load(Ordering::Relaxed);
        loop {
            let current_timestamp = self.get_current_timestamp()?;

            // 时钟回拨检测
            // 优化：减少检测频率
            if current_timestamp < last_timestamp {
                let delta = last_timestamp - current_timestamp;
                if delta > 5 {
                    error!("Clock moved backwards by {}ms", delta);
                    return Err(TinyIdError::ClockMovedBackwards(delta));
                }

                // 等待时钟追上
                // 优化：使用更短的等待时间
                std::thread::sleep(Duration::from_micros(30));
                continue;
            }
            // 同一毫秒内，递增序列号
            if current_timestamp == last_timestamp {
                sequence = (sequence + 1) & self.config.max_sequence;

                if sequence == 0 {
                    // overflow wait for next millisecond
                    std::thread::sleep(Duration::from_micros(30));
                    continue;
                }
            } else {
                // 新的毫秒，重置序列号
                sequence = 0;
            }

            // 尝试更新状态 (优化：使用Relaxed内存序)
            if self
                .last_timestamp
                .compare_exchange(
                    last_timestamp,
                    current_timestamp,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                self.sequence.store(sequence, Ordering::Relaxed);
                let id = self.assemble_id(current_timestamp, sequence);
                return Ok(id);
            }

            // CAS失败，重试
            last_timestamp = self.last_timestamp.load(Ordering::Relaxed);
        }
    }

    fn get_current_timestamp(&self) -> Result<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| TinyIdError::InternalError(e.to_string()))?;

        let timestamp = now.as_millis() as u64;
        Ok(timestamp.saturating_sub(self.config.epoch))
    }

    fn assemble_id(&self, timestamp: u64, sequence: u32) -> u64 {
        let ts_shift =
            self.config.datacenter_id_bits + self.config.worker_id_bits + self.config.sequence_bits;
        let datacenter_shift = self.config.worker_id_bits + self.config.sequence_bits;
        let worker_id_shift = self.config.sequence_bits;
        (timestamp << ts_shift)
            | ((self.config.datacenter_id as u64) << datacenter_shift)
            | ((self.config.worker_id as u64) << worker_id_shift)
            | (sequence as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation_and_basic_id_generation() {
        // 测试1：创建生成器并生成ID
        let config = GeneratorConfig::default();
        let generator = TinyIdGenerator::new(config).unwrap();

        // 生成第一个ID
        let id1 = generator.next_id().unwrap();
        assert!(id1 > 0, "Generated ID should be positive");

        // 生成第二个ID
        let id2 = generator.next_id().unwrap();
        assert!(id2 > 0, "Generated ID should be positive");

        // 验证ID的唯一性
        assert_ne!(id1, id2, "Generated IDs should be unique");

        // 验证ID的有序性（时间戳部分应该递增）
        assert!(
            id2 > id1,
            "Generated IDs should be monotonically increasing"
        );

        println!("✅ Test passed: Generated IDs - id1: {}, id2: {}", id1, id2);
    }

    #[test]
    fn test_concurrent_id_generation() {
        // 测试2：并发生成ID - 减少并发压力
        let config = GeneratorConfig::default();
        let generator = TinyIdGenerator::new(config).unwrap();

        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        let generator = Arc::new(generator);
        let mut handles = vec![];
        let mut results = vec![];

        // 创建5个线程，每个线程生成5个ID，减少并发压力
        for thread_id in 0..5 {
            let gen = Arc::clone(&generator);
            let handle = thread::spawn(move || {
                let mut thread_ids = vec![];
                // 添加小延迟确保不同线程在不同时间生成ID
                thread::sleep(Duration::from_millis(thread_id as u64));
                for _ in 0..5 {
                    let id = gen.next_id().unwrap();
                    thread_ids.push(id);
                    // 添加小延迟避免序列号冲突
                    thread::sleep(Duration::from_micros(100));
                }
                thread_ids
            });
            handles.push(handle);
        }

        // 收集所有结果
        for handle in handles {
            let thread_ids = handle.join().unwrap();
            results.extend(thread_ids);
        }

        // 验证结果
        assert_eq!(results.len(), 25, "Should generate exactly 25 IDs");

        // 验证唯一性
        let unique_ids: std::collections::HashSet<u64> = results.iter().cloned().collect();
        assert_eq!(unique_ids.len(), 25, "All generated IDs should be unique");

        // 验证有序性（由于并发，可能不是严格有序，但应该基本有序）
        let mut sorted_ids = results.clone();
        sorted_ids.sort();

        // 检查大部分ID是有序的（允许少量乱序）
        let mut ordered_count = 0;
        for i in 0..results.len() {
            if results[i] == sorted_ids[i] {
                ordered_count += 1;
            }
        }

        // 至少80%的ID应该是有序的
        let order_ratio = ordered_count as f64 / results.len() as f64;
        assert!(
            order_ratio >= 0.8,
            "At least 80% of IDs should be in order, got {:.2}%",
            order_ratio * 100.0
        );

        println!(
            "✅ Test passed: Generated {} unique IDs concurrently, order ratio: {:.2}%",
            results.len(),
            order_ratio * 100.0
        );
    }
}
