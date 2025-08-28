# TinyID OpenTelemetry Tracing 最佳实践

## 概述

本项目已集成 OpenTelemetry 分布式追踪系统，实现了以下核心功能：

### 1. 统一 Tracing 入口

- 在 `main` 函数中初始化 tracing 和 OpenTelemetry
- 作为全局 subscriber，支持结构化日志和链路追踪

### 2. HTTP Trace 传播

- 自动提取/注入 trace context
- 支持 W3C TraceContext、Jaeger、Zipkin 等标准
- 链路 ID 在上下游服务间自动传递

### 3. Metrics 与 Tracing 解耦

- Tracing 专注于链路追踪
- Metrics 独立暴露 Prometheus 指标
- 清晰的职责分离

### 4. 错误与慢请求追踪

- 自动识别错误请求（4xx, 5xx）
- 慢请求阈值监控
- 重点记录性能问题

### 5. 采样率控制

- 生产环境可配置采样率
- 避免 trace 数据爆炸
- 支持动态调整

### 6. 异步安全

- 基于 Tokio 异步运行时
- Axum/Tracing/OpenTelemetry 全异步兼容

## 配置说明

### 环境变量配置

```bash
# 服务基本信息
SERVICE_NAME=tinyid
SERVICE_VERSION=0.1.0
ENVIRONMENT=development

# Tracing 配置
RUST_LOG=info
TRACE_SAMPLE_RATE=1.0  # 开发环境 100% 采样

# OTLP 导出器（推荐）
OTLP_ENDPOINT=http://localhost:4317

# 或者使用 Jaeger（兼容 OTLP）
# OTLP_ENDPOINT=http://localhost:14268/api/traces

# 日志格式
CONSOLE_OUTPUT=true
JSON_FORMAT=true

# Metrics 服务器
METRICS_ADDRESS=0.0.0.0
METRICS_PORT=9090
```

### 生产环境建议

```bash
# 生产环境配置
ENVIRONMENT=production
TRACE_SAMPLE_RATE=0.1  # 10% 采样率
RUST_LOG=warn
JSON_FORMAT=true

# 使用云服务
OTLP_ENDPOINT=https://api.datadoghq.com/api/v1/traces
# 或者
OTLP_ENDPOINT=https://trace-api.newrelic.com/trace/v1
```

## 代码实现要点

### 1. 统一初始化

```rust
// main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化环境变量
    tinyid::init_env();
    
    // 2. 初始化 tracing（统一入口）
    let tracing_cleanup = metric::init_tracing()?;
    
    // 3. 初始化 metrics 系统
    let (metrics_server, app_metrics) = metric::init_metrics()?;
    
    // ... 应用逻辑
    
    // 4. 清理资源
    tracing_cleanup.cleanup();
    Ok(())
}
```

### 2. HTTP 中间件

```rust
// 自动 trace 传播
let app = Router::new()
    .route("/api/id", get(generate_id_handler))
    .layer(create_tracing_layer_with_config(TracingConfig::default()));
```

### 3. 手动 Span 创建

```rust
use tracing::{info_span, instrument};

// 使用 instrument 宏（推荐）
#[instrument(skip(self))]
async fn generate_id(&self) -> Result<u64> {
    info!("Generating new ID");
    // 业务逻辑
    Ok(id)
}

// 手动创建 span
async fn process_request(request_id: &str) {
    let span = info_span!("process_request", request_id = request_id);
    let _enter = span.enter();
    
    info!("Processing request");
    // 业务逻辑
}
```

## 监控系统集成

### Jaeger

```bash
# 启动 Jaeger
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 14268:14268 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest

# 设置环境变量
export OTLP_ENDPOINT=http://localhost:4317
```

### Prometheus + Grafana

```bash
# Prometheus 配置
scrape_configs:
  - job_name: 'tinyid'
    static_configs:
      - targets: ['localhost:9090']

# 访问指标
curl http://localhost:9090/metrics
```

### DataDog

```bash
# 设置 DataDog OTLP
export OTLP_ENDPOINT=https://api.datadoghq.com/api/v1/traces
export DD_API_KEY=your_api_key
```

## 最佳实践

### 1. Span 命名规范

- HTTP 请求：`"GET /api/users"`
- 数据库操作：`"db.query.users"`
- 外部调用：`"http.client.payment_service"`
- 业务逻辑：`"business.generate_id"`

### 2. 属性添加

```rust
use tracing::info_span;
use opentelemetry::KeyValue;

let span = info_span!(
    "database_query",
    db.operation = "SELECT",
    db.table = "users",
    user.id = %user_id
);
```

### 3. 错误处理

```rust
use tracing::{error, warn};

match result {
    Ok(value) => {
        info!(result = %value, "Operation completed successfully");
        Ok(value)
    }
    Err(e) => {
        error!(error = %e, "Operation failed");
        // 错误会自动标记 span 状态
        Err(e)
    }
}
```

### 4. 采样策略

- 开发环境：100% 采样 (`TRACE_SAMPLE_RATE=1.0`)
- 测试环境：50% 采样 (`TRACE_SAMPLE_RATE=0.5`) 
- 生产环境：1-10% 采样 (`TRACE_SAMPLE_RATE=0.1`)

### 5. 性能优化

- 使用 `instrument` 宏而非手动 span
- 避免在热路径创建过多 span
- 合理设置日志级别
- 定期清理 trace 数据

## 故障排查

### 1. Trace 数据未发送

检查：
- OTLP_ENDPOINT 配置是否正确
- 网络连接是否正常
- 采样率是否过低

### 2. 性能影响

优化措施：
- 降低采样率
- 减少 span 属性数量
- 使用异步导出器

### 3. 链路断裂

常见原因：
- 中间件顺序错误
- 手动 span 管理错误
- 跨服务调用未传递 context

## 扩展功能

### 自定义 Exporter

```rust
// 自定义导出器
let exporter = CustomExporter::new();
let tracer_provider = TracerProvider::builder()
    .with_batch_exporter(exporter, Tokio)
    .build();
```

### 动态采样

```rust
// 基于请求特征的动态采样
let sampler = if is_important_request(&request) {
    Sampler::AlwaysOn
} else {
    Sampler::TraceIdRatioBased(0.1)
};
```

## 总结

本实现提供了完整的 OpenTelemetry 分布式追踪解决方案，具备：

✅ 统一 tracing 入口  
✅ HTTP trace 传播  
✅ 错误和慢请求追踪  
✅ Metrics 与 tracing 解耦  
✅ 采样率控制  
✅ 异步安全  
✅ 链路 ID 透传  

通过合理配置和使用，可以为微服务架构提供强大的可观测性支持。
