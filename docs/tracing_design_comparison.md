# Tracing 设计方式对比分析

## 概述

在Rust生态系统中，有三种主要的tracing实现方式。本文档分析了每种方式的优缺点，并推荐最符合标准的设计。

## 三种设计方式

### 1. 混合双层设计（当前实现）

```rust
// middleware.rs 中的实现
let tracer = opentelemetry::global::tracer("tinyid");
let otel_span = tracer.span_builder("http_request").start();
let cx = parent_cx.with_span(otel_span);

let tracing_span = tracing::info_span!("http_request", ...);
tracing_span.set_parent(cx.clone());  // 关联两个span系统
```

**特点：**
- ✅ 同时使用OpenTelemetry和tracing
- ✅ 完全控制span属性
- ❌ 复杂的双层span管理
- ❌ 容易出现span聚合问题
- ❌ 代码复杂度高
- ❌ 性能开销大

**问题：**
- 创建两套span系统
- span父子关系复杂
- 并发请求容易产生span聚合
- 维护成本高

### 2. Pure OpenTelemetry 设计

```rust
use opentelemetry::trace::{Tracer, SpanKind};

let tracer = opentelemetry::global::tracer("tinyid");
let span = tracer
    .span_builder("http_request")
    .with_kind(SpanKind::Server)
    .with_attributes([
        KeyValue::new("http.method", method),
        KeyValue::new("http.route", path),
    ])
    .start_with_context(&tracer, &parent_cx);
```

**特点：**
- ✅ 直接使用OpenTelemetry API
- ✅ 完全符合OpenTelemetry规范
- ✅ 性能最优
- ❌ 失去tracing crate的便利性
- ❌ 不能使用#[instrument]宏
- ❌ 结构化日志支持有限

**适用场景：**
- 性能要求极高的系统
- 需要精确控制span行为
- 不需要结构化日志

### 3. Pure Tracing + OpenTelemetryLayer 设计（🏆 推荐）

```rust
// 在 traces.rs 中配置
let otel_layer = tracing_opentelemetry::layer()
    .with_tracer(tracer_provider.tracer("tinyid"));

// 在业务代码中
let span = tracing::info_span!(
    "http_request",
    "http.method" = %method,
    "http.route" = %path,
    "request.id" = %request_id,
);
let _guard = span.enter();
```

**特点：**
- ✅ 使用Rust惯用的tracing crate
- ✅ 自动OpenTelemetry导出
- ✅ 支持#[instrument]宏
- ✅ 优秀的结构化日志
- ✅ 每个请求独立span
- ✅ 代码简洁易维护
- ✅ 符合OpenTelemetry语义约定

## 推荐设计：Pure Tracing + OpenTelemetryLayer

### 核心原理

1. **单一span系统**：只使用tracing span
2. **自动导出**：OpenTelemetryLayer自动处理OpenTelemetry导出
3. **语义约定**：遵循OpenTelemetry语义约定命名字段
4. **独立性**：每个请求有独立的span，避免聚合

### 实现要点

#### 1. 初始化配置

```rust
// libs/shared/src/traces_optimal.rs
let otel_layer = tracing_opentelemetry::layer()
    .with_error_records_to_exceptions(true)  // 错误自动转exception
    .with_location(true)                     // 包含代码位置
    .with_targets(true)                      // 包含target信息
    .with_tracer(tracer_provider.tracer("tinyid"));

let registry = Registry::default()
    .with(env_filter)
    .with(otel_layer);  // 关键：OpenTelemetryLayer负责导出
```

#### 2. 中间件实现

```rust
// middleware_optimal.rs
let span = tracing::info_span!(
    "http_request",
    // OpenTelemetry 语义约定
    "http.method" = %method,
    "http.route" = %path,
    "http.url" = %uri,
    "http.user_agent" = %user_agent,
    "http.response.status_code" = tracing::field::Empty,
    
    // 自定义字段
    "request.id" = %request_id,
    "service.name" = "tinyid",
    "span.kind" = "server",
);

let _guard = span.enter();
```

#### 3. Span事件配置

```rust
pub enum SpanEventsConfig {
    None,                           // 生产环境最高性能
    EnterExit,                     // 推荐：只记录进入/退出
    All,                           // 开发环境：所有事件
    Custom(FmtSpan),               // 自定义配置
}
```

### 解决span聚合问题

推荐设计通过以下方式解决span聚合问题：

1. **独立请求ID**：每个请求生成唯一ID
2. **正确的span配置**：使用`ENTER|EXIT`而非`CLOSE`
3. **语义约定字段**：使用标准化字段名
4. **避免双层span**：只使用tracing span系统

### 性能对比

| 设计方式 | CPU开销 | 内存开销 | 复杂度 | 维护性 |
|---------|---------|----------|--------|--------|
| 混合双层 | 高      | 高       | 高     | 低     |
| Pure OTEL| 最低    | 最低     | 中     | 中     |
| Pure Tracing| 低   | 低       | 低     | 高     |

### 配置建议

#### 开发环境
```rust
OptimalTracingConfig {
    sample_rate: 1.0,
    span_events: SpanEventsConfig::All,
    console_output: true,
    json_format: false,  // 便于阅读
    ..Default::default()
}
```

#### 生产环境
```rust
OptimalTracingConfig {
    sample_rate: 0.1,    // 10%采样
    span_events: SpanEventsConfig::EnterExit,
    console_output: true,
    json_format: true,   // 便于解析
    ..Default::default()
}
```

## 迁移指南

### 从当前实现迁移到推荐设计

1. **替换中间件**：
   ```rust
   // 旧的
   .layer(axum::middleware::from_fn(tracing_middleware_with_config))
   
   // 新的
   .layer(axum::middleware::from_fn(optimal_tracing_middleware))
   ```

2. **更新初始化**：
   ```rust
   // 旧的
   let _cleanup = shared::init_tracing()?;
   
   // 新的
   let _cleanup = shared::init_optimal_tracing()?;
   ```

3. **更新业务代码**：
   ```rust
   // 旧的
   #[tracing::instrument(skip(self))]
   
   // 新的（保持不变，但字段名使用语义约定）
   #[tracing::instrument(skip(self), fields(
       "operation.name" = "generate_id",
       "service.name" = "tinyid"
   ))]
   ```

## 总结

**推荐使用 Pure Tracing + OpenTelemetryLayer 设计**，因为它：

1. **符合Rust生态习惯**：使用标准的tracing crate
2. **自动OpenTelemetry集成**：无需手动管理两套span
3. **解决并发问题**：每个请求独立span，避免聚合
4. **性能优异**：单一span系统，开销最小
5. **易于维护**：代码简洁，符合标准

这种设计是当前Rust社区的最佳实践，被广泛应用于各大项目中。
