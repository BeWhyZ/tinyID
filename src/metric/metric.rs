use std::sync::Once;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
};

static INIT: Once = Once::new();

pub fn init_logs() {
    // 使用 Once 确保只初始化一次
    INIT.call_once(|| {
        // 初始化控制台日志输出
        let subscriber = tracing_subscriber::fmt()
            .json() // 使用JSON格式，包含trace_id和span_id
            .with_env_filter(EnvFilter::from_env("RUST_LOG"))
            .with_span_events(FmtSpan::CLOSE) // 记录span开始和结束
            .with_timer(fmt::time::UtcTime::rfc_3339())
            .with_target(false) // 隐藏目标模块路径
            .with_level(true) // 显示日志级别
            .with_ansi(true)
            .with_current_span(true) // 显示当前span信息
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global subscriber");
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
    use tracing::{debug, error, info, instrument, warn, Span};

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
