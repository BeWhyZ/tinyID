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

/// Tracing 配置结构
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: String,
    /// 服务环境 (dev, staging, prod)
    pub environment: String,
    /// 采样率 (0.0-1.0)
    pub sample_rate: f64,
    /// OTLP collector endpoint (支持 Jaeger, DataDog, New Relic 等)
    pub otlp_endpoint: Option<String>,
    /// 日志级别
    pub log_level: String,
    /// 是否启用控制台输出
    pub console_output: bool,
    /// 是否启用JSON格式
    pub json_format: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "tinyid".to_string()),
            service_version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            sample_rate: env::var("TRACE_SAMPLE_RATE")
                .unwrap_or_else(|_| "1.0".to_string())
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
        }
    }
}

/// 初始化 OpenTelemetry tracer
fn init_opentelemetry(
    config: &TracingConfig,
) -> Result<opentelemetry_sdk::trace::SdkTracerProvider> {
    use opentelemetry_otlp::WithExportConfig;

    // 创建资源描述符
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
            KeyValue::new("service.environment", config.environment.clone()),
            KeyValue::new("service.instance.id", uuid::Uuid::new_v4().to_string()),
        ])
        .build();

    // 配置采样器
    let sampler = Sampler::AlwaysOn;

    // 创建 tracer provider
    let tracer_provider = if let Some(otlp_endpoint) = &config.otlp_endpoint {
        info!("Initializing OTLP tracer with endpoint: {}", otlp_endpoint);

        // 创建 OTLP HTTP exporter
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build()
            .expect("Failed to create span exporter");

        // 使用 batch exporter
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
            .with_batch_exporter(exporter)
            .with_sampler(sampler)
            .build()
    };

    Ok(tracer_provider)
}

/// 统一的 tracing 初始化入口
pub fn init_tracing() -> Result<TracingCleanup> {
    init_tracing_with_config(TracingConfig::default())
}

/// 使用自定义配置初始化 tracing
pub fn init_tracing_with_config(config: TracingConfig) -> Result<TracingCleanup> {
    let mut cleanup = TracingCleanup::default();

    try_init_tracing(&config, &mut cleanup)?;

    Ok(cleanup)
}

fn try_init_tracing(config: &TracingConfig, cleanup: &mut TracingCleanup) -> Result<()> {
    info!("Initializing tracing with config: {:?}", config);

    // 1. 初始化 OpenTelemetry
    let tracer_provider = init_opentelemetry(config)?;
    cleanup.tracer_provider = Some(tracer_provider.clone());

    // 2. 创建 OpenTelemetry layer
    let trace_layer = tracing_opentelemetry::layer()
        .with_error_records_to_exceptions(true)
        .with_tracer(tracer_provider.tracer("tinyid"));

    // 3. 创建环境过滤器
    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&config.log_level))?;
    // 4. 构建 subscriber
    let registry = Registry::default().with(env_filter).with(trace_layer);

    // 5. 添加控制台输出层（如果启用）
    if config.console_output {
        if config.json_format {
            let fmt_layer = fmt::layer()
                .json()
                .with_span_events(FmtSpan::NEW | FmtSpan::ENTER | FmtSpan::EXIT) // 只显示进入和退出，不显示CLOSE汇总
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_target(false)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true);

            registry.with(fmt_layer).init();
        } else {
            let fmt_layer = fmt::layer()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_target(false)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true);

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
        "Tracing initialized successfully"
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
            if let Err(e) = provider.shutdown() {
                error!("Failed to shutdown tracer provider: {:?}", e);
            } else {
                info!("Tracer provider shutdown successfully");
            }
        }
    }
}

/// 兼容性函数，保持旧接口
pub fn init_logs() {
    INIT.call_once(|| {
        if let Err(e) = init_tracing() {
            error!("Failed to initialize tracing: {}", e);
        }
    });
}

// 示例函数：使用instrument宏自动创建span
#[tracing::instrument]
fn generate_id_with_span() -> u64 {
    info!("Starting ID generation");

    // 模拟一些工作
    std::thread::sleep(std::time::Duration::from_millis(10));

    let id = 123456789;
    info!("Generated ID: {}", id);
    id
}

// 示例函数：手动创建span
fn process_request_with_manual_span(request_id: &str) {
    let span = tracing::info_span!("process_request", request_id = request_id);
    let _enter = span.enter();

    info!("Processing request");

    // 创建嵌套span
    {
        let nested_span = tracing::info_span!("validate_request");
        let _nested_enter = nested_span.enter();
        info!("Validating request data");
    }

    {
        let nested_span = tracing::info_span!("generate_response");
        let _nested_enter = nested_span.enter();
        info!("Generating response");
    }

    info!("Request processed successfully");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{debug, error, info, instrument, warn};

    use crate::init_env;

    #[test]
    fn test_init_logs() {
        init_env();
        init_logs();

        // 创建根span
        let span = tracing::info_span!("test_operation", operation = "log_test");
        let _enter = span.enter();

        info!("this is a info log");
        debug!("this is a debug log");
        warn!("this is a warn log");
        error!("this is a error log");

        // 创建嵌套span
        let nested_span = tracing::info_span!("nested_operation", detail = "nested_test");
        let _nested_enter = nested_span.enter();

        info!("this is a nested info log");
        debug!("this is a nested debug log");
    }

    #[test]
    #[instrument]
    fn test_instrumented_function() {
        init_env();
        init_logs();

        info!("this is an instrumented function log");
        debug!("this is an instrumented debug log");
    }

    #[test]
    fn test_span_hierarchy() {
        init_env();
        init_logs();

        // 测试span层次结构
        let root_span = tracing::info_span!("root_operation", operation_type = "test");
        let _root_enter = root_span.enter();

        info!("Root span log");

        // 嵌套span 1
        {
            let child_span = tracing::info_span!("child_operation", child_id = 1);
            let _child_enter = child_span.enter();
            info!("Child span 1 log");

            // 更深层的嵌套
            {
                let grandchild_span = tracing::info_span!("grandchild_operation", level = 3);
                let _grandchild_enter = grandchild_span.enter();
                info!("Grandchild span log");
            }
        }

        // 嵌套span 2
        {
            let child_span = tracing::info_span!("child_operation", child_id = 2);
            let _child_enter = child_span.enter();
            info!("Child span 2 log");
        }

        info!("Back to root span");
    }

    #[test]
    fn test_instrumented_functions() {
        init_env();
        init_logs();

        // 测试使用instrument宏的函数
        let id = generate_id_with_span();
        assert_eq!(id, 123456789);

        // 测试手动创建span的函数
        process_request_with_manual_span("req-123");
    }
}
