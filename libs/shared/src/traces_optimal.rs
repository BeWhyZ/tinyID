use std::env;
use std::sync::Once;

use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider, Tracer};
use opentelemetry_sdk::Resource;
use tracing::{error, info, warn, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer, Registry,
};

static INIT: Once = Once::new();

/// 最佳实践的 Tracing 配置结构
#[derive(Debug, Clone)]
pub struct OptimalTracingConfig {
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: String,
    /// 服务环境 (dev, staging, prod)
    pub environment: String,
    /// 采样率 (0.0-1.0)
    pub sample_rate: f64,
    /// OTLP collector endpoint
    pub otlp_endpoint: Option<String>,
    /// 日志级别
    pub log_level: String,
    /// 是否启用控制台输出
    pub console_output: bool,
    /// 是否启用JSON格式
    pub json_format: bool,
    /// span事件配置
    pub span_events: SpanEventsConfig,
}

/// Span 事件配置
#[derive(Debug, Clone)]
pub enum SpanEventsConfig {
    /// 不记录span事件（最高性能）
    None,
    /// 只记录span进入/退出（推荐用于生产环境）
    EnterExit,
    /// 记录所有span事件（适合开发/调试）
    All,
    /// 自定义配置
    Custom(FmtSpan),
}

impl Default for OptimalTracingConfig {
    fn default() -> Self {
        // 根据环境智能选择默认配置
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let span_events = match environment.as_str() {
            "production" => SpanEventsConfig::EnterExit, // 生产环境最小化输出
            "staging" => SpanEventsConfig::EnterExit,
            _ => SpanEventsConfig::All, // 开发环境详细输出
        };

        Self {
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "tinyid".to_string()),
            service_version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            environment,
            sample_rate: env::var("TRACE_SAMPLE_RATE")
                .unwrap_or_else(|_| match env::var("ENVIRONMENT").as_deref() {
                    Ok("production") => "0.1".to_string(), // 生产环境10%采样
                    Ok("staging") => "0.5".to_string(),    // 测试环境50%采样
                    _ => "1.0".to_string(),                // 开发环境100%采样
                })
                .parse()
                .unwrap_or(1.0),
            otlp_endpoint: env::var("OTLP_ENDPOINT").ok(),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            console_output: env::var("CONSOLE_OUTPUT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            json_format: env::var("JSON_FORMAT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            span_events,
        }
    }
}

/// 初始化 OpenTelemetry tracer（最佳实践版本）
fn init_opentelemetry_optimal(
    config: &OptimalTracingConfig,
) -> Result<opentelemetry_sdk::trace::SdkTracerProvider> {
    use opentelemetry_otlp::WithExportConfig;

    // 创建资源描述符（遵循OpenTelemetry语义约定）
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
            KeyValue::new("deployment.environment", config.environment.clone()),
            KeyValue::new("service.instance.id", uuid::Uuid::new_v4().to_string()),
            // 添加更多语义约定属性
            KeyValue::new("telemetry.sdk.name", "opentelemetry"),
            KeyValue::new("telemetry.sdk.language", "rust"),
            KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    // 配置智能采样器
    let sampler = if config.sample_rate >= 1.0 {
        Sampler::AlwaysOn
    } else if config.sample_rate <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.sample_rate)
    };

    // 创建 tracer provider
    let tracer_provider = if let Some(otlp_endpoint) = &config.otlp_endpoint {
        info!("Initializing OTLP tracer with endpoint: {}", otlp_endpoint);

        // 创建 OTLP exporter（支持gRPC和HTTP）
        let exporter =
            if otlp_endpoint.starts_with("http://") || otlp_endpoint.starts_with("https://") {
                // HTTP exporter
                SpanExporter::builder()
                    .with_http()
                    .with_endpoint(otlp_endpoint)
                    .build()
                    .expect("Failed to create HTTP span exporter")
            } else {
                // gRPC exporter (默认)
                SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(otlp_endpoint)
                    .build()
                    .expect("Failed to create gRPC span exporter")
            };

        // 使用 batch exporter 提高性能
        SdkTracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(exporter)
            .with_sampler(sampler)
            .build()
    } else {
        info!("No external tracing endpoint configured, using stdout exporter for development");
        // 开发环境下使用 stdout exporter
        let exporter = opentelemetry_stdout::SpanExporter::default();

        SdkTracerProvider::builder()
            .with_resource(resource)
            .with_simple_exporter(exporter) // 开发环境使用simple exporter降低延迟
            .with_sampler(sampler)
            .build()
    };

    Ok(tracer_provider)
}

/// 最佳实践的 tracing 初始化
pub fn init_optimal_tracing() -> Result<TracingCleanup> {
    init_optimal_tracing_with_config(OptimalTracingConfig::default())
}

/// 使用自定义配置初始化最佳实践 tracing
pub fn init_optimal_tracing_with_config(config: OptimalTracingConfig) -> Result<TracingCleanup> {
    let mut cleanup = TracingCleanup::default();

    try_init_optimal_tracing(&config, &mut cleanup)?;

    Ok(cleanup)
}

fn try_init_optimal_tracing(
    config: &OptimalTracingConfig,
    cleanup: &mut TracingCleanup,
) -> Result<()> {
    info!("Initializing optimal tracing with config: {:?}", config);

    // 1. 初始化 OpenTelemetry
    let tracer_provider = init_opentelemetry_optimal(config)?;
    cleanup.tracer_provider = Some(tracer_provider.clone());

    // 2. 创建 OpenTelemetry layer（关键：这是推荐的方式）
    let otel_layer = tracing_opentelemetry::layer()
        .with_error_records_to_exceptions(true) // 错误自动转换为exceptions
        .with_location(true) // 包含代码位置信息
        .with_targets(true) // 包含target信息
        .with_tracer(tracer_provider.tracer("tinyid"));

    // 3. 创建环境过滤器
    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&config.log_level))?;

    // 4. 构建 subscriber registry
    let registry = Registry::default().with(env_filter).with(otel_layer); // OpenTelemetry layer 负责span导出

    // 5. 添加控制台输出层（如果启用）
    if config.console_output {
        let span_events = match &config.span_events {
            SpanEventsConfig::None => FmtSpan::NONE,
            SpanEventsConfig::EnterExit => FmtSpan::ENTER | FmtSpan::EXIT,
            SpanEventsConfig::All => FmtSpan::NEW | FmtSpan::ENTER | FmtSpan::EXIT | FmtSpan::CLOSE,
            SpanEventsConfig::Custom(events) => *events,
        };

        if config.json_format {
            let fmt_layer = fmt::layer()
                .json()
                .with_span_events(span_events)
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_target(false)
                .with_level(true)
                .with_thread_ids(false) // 生产环境可关闭降低开销
                .with_thread_names(false)
                .with_file(false) // 不输出文件名降低开销
                .with_line_number(false); // 不输出行号降低开销

            registry.with(fmt_layer).init();
        } else {
            let fmt_layer = fmt::layer()
                .with_span_events(span_events)
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_target(false)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false);

            registry.with(fmt_layer).init();
        }
    } else {
        registry.init();
    }

    info!(
        service_name = %config.service_name,
        service_version = %config.service_version,
        environment = %config.environment,
        sample_rate = %config.sample_rate,
        otlp_endpoint = ?config.otlp_endpoint,
        "Optimal tracing initialized successfully"
    );

    // 设置全局 tracer provider
    global::set_tracer_provider(tracer_provider.clone());

    Ok(())
}

/// 清理资源的结构体
#[derive(Default)]
pub struct TracingCleanup {
    tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
}

impl TracingCleanup {
    /// 执行清理操作
    pub fn cleanup(self) {
        if let Some(provider) = self.tracer_provider {
            // 强制刷新所有待发送的span
            if let Err(e) = provider.force_flush() {
                error!("Failed to flush tracer provider: {:?}", e);
            }

            // 关闭tracer provider
            if let Err(e) = provider.shutdown() {
                error!("Failed to shutdown tracer provider: {:?}", e);
            } else {
                info!("Tracer provider shutdown successfully");
            }
        }

        // 关闭全局tracer provider
        global::shutdown_tracer_provider();
    }
}

/// 最佳实践：兼容性函数
pub fn init_logs() {
    INIT.call_once(|| {
        if let Err(e) = init_optimal_tracing() {
            error!("Failed to initialize optimal tracing: {}", e);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{debug, error, info, instrument, warn};

    #[test]
    fn test_optimal_tracing_config() {
        // 测试默认配置
        let config = OptimalTracingConfig::default();
        assert_eq!(config.service_name, "tinyid");
        assert!(config.sample_rate >= 0.0 && config.sample_rate <= 1.0);
    }

    #[test]
    fn test_init_optimal_logs() {
        let config = OptimalTracingConfig {
            console_output: true,
            json_format: false,
            span_events: SpanEventsConfig::EnterExit,
            ..OptimalTracingConfig::default()
        };

        let cleanup = init_optimal_tracing_with_config(config).expect("Failed to init tracing");

        // 创建测试span
        let span = tracing::info_span!("test_operation", test_field = "test_value");
        let _enter = span.enter();

        info!("this is a test info log");
        debug!("this is a test debug log");
        warn!("this is a test warn log");
        error!("this is a test error log");

        // 清理
        cleanup.cleanup();
    }

    #[test]
    #[instrument]
    fn test_instrumented_function() {
        let config = OptimalTracingConfig::default();
        let cleanup = init_optimal_tracing_with_config(config).expect("Failed to init tracing");

        info!("this is an instrumented function log");
        debug!("this is an instrumented debug log");

        cleanup.cleanup();
    }
}
