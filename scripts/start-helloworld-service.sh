#!/bin/bash

# 启动 HelloWorld Service 脚本

set -e

echo "Starting HelloWorld Service..."

# 设置环境变量
export APP_NAME="helloworld-service"
export APP_VERSION="0.1.0"
export APP_HOST="0.0.0.0"
export APP_PORT="8080"
export APP_ENVIRONMENT="development"
export APP_DEPENDENCIES__0__NAME="user-service"
export APP_DEPENDENCIES__0__HOST="127.0.0.1"
export APP_DEPENDENCIES__0__PORT="9001"
export APP_DEPENDENCIES__0__PROTOCOL="grpc"
export APP_TRACING__ENABLED="true"
export APP_TRACING__SERVICE_NAME="helloworld-service"
export APP_TRACING__SAMPLE_RATE="1.0"

# 构建服务
echo "Building helloworld service..."
cargo build --release --bin server --manifest-path services/helloworld/Cargo.toml

# 启动服务
echo "Starting helloworld service on port $APP_PORT..."
exec ./target/release/server
