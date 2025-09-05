#!/bin/bash

# 启动 User Service 脚本

set -e

echo "Starting User Service..."

# 设置环境变量
export APP_NAME="user-service"
export APP_VERSION="0.1.0"
export APP_HOST="0.0.0.0"
export APP_PORT="8081"
export APP_GRPC_PORT="9001"
export APP_ENVIRONMENT="development"
export APP_TRACING__ENABLED="true"
export APP_TRACING__SERVICE_NAME="user-service"
export APP_TRACING__SAMPLE_RATE="1.0"

# 构建服务
echo "Building user service..."
cargo build --release --bin server --manifest-path services/user/Cargo.toml

# 启动服务
echo "Starting user service on port $APP_GRPC_PORT..."
exec ./target/release/server
