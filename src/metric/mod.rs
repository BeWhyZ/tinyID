pub mod metrics;

// 导出 metrics 功能
pub use metrics::{
    init_metrics, init_metrics_with_config, AppMetrics, MetricsConfig, MetricsServer,
};
