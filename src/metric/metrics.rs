use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::json;
use tokio::net::TcpListener;
use tracing::info;

/// Metrics 配置
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Prometheus 端点地址
    pub address: String,
    /// Prometheus 端点端口
    pub port: u16,
    /// 指标路径
    pub metrics_path: String,
    /// 健康检查路径
    pub health_path: String,
    /// 是否启用详细指标
    pub enable_detailed_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            address: std::env::var("METRICS_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("METRICS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .unwrap_or(9090),
            metrics_path: std::env::var("METRICS_PATH").unwrap_or_else(|_| "/metrics".to_string()),
            health_path: std::env::var("HEALTH_PATH").unwrap_or_else(|_| "/health".to_string()),
            enable_detailed_metrics: std::env::var("ENABLE_DETAILED_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

/// 应用程序指标
#[derive(Debug, Clone)]
pub struct AppMetrics {
    /// 服务启动时间
    pub start_time: Instant,
    /// 总请求数
    pub total_requests: Arc<std::sync::atomic::AtomicU64>,
    /// 成功请求数
    pub successful_requests: Arc<std::sync::atomic::AtomicU64>,
    /// 失败请求数
    pub failed_requests: Arc<std::sync::atomic::AtomicU64>,
    /// 生成的 ID 总数
    pub generated_ids: Arc<std::sync::atomic::AtomicU64>,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            total_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            successful_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            failed_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            generated_ids: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            avg_response_time_ms: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

impl AppMetrics {
    /// 增加请求计数
    pub fn increment_request(&self) {
        self.total_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// 记录成功请求
    pub fn record_success(&self, response_time_ms: u64) {
        self.successful_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.update_avg_response_time(response_time_ms);
    }

    /// 记录失败请求
    pub fn record_failure(&self, response_time_ms: u64) {
        self.failed_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.update_avg_response_time(response_time_ms);
    }

    /// 增加生成的 ID 计数
    pub fn increment_generated_ids(&self) {
        self.generated_ids
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// 更新平均响应时间
    fn update_avg_response_time(&self, response_time_ms: u64) {
        // 简单的移动平均算法
        let current_avg = self
            .avg_response_time_ms
            .load(std::sync::atomic::Ordering::Relaxed);
        let total_requests = self
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);

        if total_requests > 0 {
            let new_avg = (current_avg * (total_requests - 1) + response_time_ms) / total_requests;
            self.avg_response_time_ms
                .store(new_avg, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// 获取运行时间（秒）
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

/// Metrics 服务器
pub struct MetricsServer {
    config: MetricsConfig,
    metrics: Arc<AppMetrics>,
}

impl MetricsServer {
    /// 创建新的 metrics 服务器
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(AppMetrics::default()),
        }
    }

    /// 获取指标实例的引用
    pub fn metrics(&self) -> Arc<AppMetrics> {
        Arc::clone(&self.metrics)
    }

    /// 启动 metrics 服务器
    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        info!("Metrics server listening on {}", addr);

        let app = self.create_router();

        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Metrics server error: {}", e))?;

        Ok(())
    }

    /// 带优雅关闭的启动方式
    pub async fn start_with_shutdown(
        &self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        info!("Metrics server listening on {}", addr);

        let app = self.create_router();

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| anyhow::anyhow!("Metrics server error: {}", e))?;

        Ok(())
    }

    /// 创建路由器
    fn create_router(&self) -> Router {
        let metrics = Arc::clone(&self.metrics);

        Router::new()
            .route(&self.config.metrics_path, get(metrics_handler))
            .route(&self.config.health_path, get(health_handler))
            .with_state(metrics)
    }
}

/// Prometheus 格式的指标处理器
async fn metrics_handler(State(metrics): State<Arc<AppMetrics>>) -> impl IntoResponse {
    let total_requests = metrics
        .total_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    let successful_requests = metrics
        .successful_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    let failed_requests = metrics
        .failed_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    let generated_ids = metrics
        .generated_ids
        .load(std::sync::atomic::Ordering::Relaxed);
    let avg_response_time = metrics
        .avg_response_time_ms
        .load(std::sync::atomic::Ordering::Relaxed);
    let uptime = metrics.uptime_seconds();

    // 生成 Prometheus 格式的指标
    let prometheus_metrics = format!(
        r#"# HELP tinyid_requests_total Total number of HTTP requests
# TYPE tinyid_requests_total counter
tinyid_requests_total {{}} {}

# HELP tinyid_requests_successful_total Total number of successful HTTP requests  
# TYPE tinyid_requests_successful_total counter
tinyid_requests_successful_total {{}} {}

# HELP tinyid_requests_failed_total Total number of failed HTTP requests
# TYPE tinyid_requests_failed_total counter
tinyid_requests_failed_total {{}} {}

# HELP tinyid_ids_generated_total Total number of IDs generated
# TYPE tinyid_ids_generated_total counter
tinyid_ids_generated_total {{}} {}

# HELP tinyid_response_time_avg_ms Average response time in milliseconds
# TYPE tinyid_response_time_avg_ms gauge
tinyid_response_time_avg_ms {{}} {}

# HELP tinyid_uptime_seconds Service uptime in seconds
# TYPE tinyid_uptime_seconds gauge
tinyid_uptime_seconds {{}} {}

# HELP tinyid_success_rate Request success rate
# TYPE tinyid_success_rate gauge
tinyid_success_rate {{}} {}
"#,
        total_requests,
        successful_requests,
        failed_requests,
        generated_ids,
        avg_response_time,
        uptime,
        if total_requests > 0 {
            successful_requests as f64 / total_requests as f64
        } else {
            0.0
        }
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(prometheus_metrics)
        .unwrap()
}

/// 健康检查处理器
async fn health_handler(State(metrics): State<Arc<AppMetrics>>) -> impl IntoResponse {
    let uptime = metrics.uptime_seconds();
    let total_requests = metrics
        .total_requests
        .load(std::sync::atomic::Ordering::Relaxed);

    let health_status = json!({
        "status": "healthy",
        "uptime_seconds": uptime,
        "total_requests": total_requests,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "tinyid-metrics",
        "version": env!("CARGO_PKG_VERSION")
    });

    axum::Json(health_status)
}

/// 初始化 metrics 系统
pub fn init_metrics() -> Result<(MetricsServer, Arc<AppMetrics>)> {
    init_metrics_with_config(MetricsConfig::default())
}

/// 使用自定义配置初始化 metrics 系统
pub fn init_metrics_with_config(config: MetricsConfig) -> Result<(MetricsServer, Arc<AppMetrics>)> {
    info!("Initializing metrics system with config: {:?}", config);

    let server = MetricsServer::new(config);
    let metrics = server.metrics();

    info!("Metrics system initialized successfully");

    Ok((server, metrics))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_app_metrics() {
        let metrics = AppMetrics::default();

        // 测试计数器
        metrics.increment_request();
        assert_eq!(
            metrics
                .total_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        metrics.record_success(100);
        assert_eq!(
            metrics
                .successful_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        metrics.record_failure(200);
        assert_eq!(
            metrics
                .failed_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        metrics.increment_generated_ids();
        assert_eq!(
            metrics
                .generated_ids
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_metrics_config() {
        let config = MetricsConfig::default();
        assert_eq!(config.address, "0.0.0.0");
        assert_eq!(config.port, 9090);
        assert_eq!(config.metrics_path, "/metrics");
        assert_eq!(config.health_path, "/health");
    }

    #[tokio::test]
    async fn test_metrics_server_creation() {
        let config = MetricsConfig::default();
        let server = MetricsServer::new(config);

        // 测试指标访问
        let metrics = server.metrics();
        metrics.increment_request();

        assert_eq!(
            metrics
                .total_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }
}
