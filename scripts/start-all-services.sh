#!/bin/bash

# 启动所有微服务的脚本

set -e

echo "Starting all microservices..."

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the project root directory"
    exit 1
fi

# 创建日志目录
mkdir -p logs

# 构建所有服务
echo "Building all services..."
cargo build --release

# 启动 User Service (后台运行)
echo "Starting User Service..."
nohup ./scripts/start-user-service.sh > logs/user-service.log 2>&1 &
USER_SERVICE_PID=$!
echo "User Service started with PID: $USER_SERVICE_PID"

# 等待 User Service 启动
echo "Waiting for User Service to start..."
sleep 3

# 检查 User Service 是否正在运行
if ! kill -0 $USER_SERVICE_PID 2>/dev/null; then
    echo "Error: User Service failed to start"
    exit 1
fi

# 启动 HelloWorld Service (后台运行)
echo "Starting HelloWorld Service..."
nohup ./scripts/start-helloworld-service.sh > logs/helloworld-service.log 2>&1 &
HELLOWORLD_SERVICE_PID=$!
echo "HelloWorld Service started with PID: $HELLOWORLD_SERVICE_PID"

# 等待 HelloWorld Service 启动
echo "Waiting for HelloWorld Service to start..."
sleep 3

# 检查 HelloWorld Service 是否正在运行
if ! kill -0 $HELLOWORLD_SERVICE_PID 2>/dev/null; then
    echo "Error: HelloWorld Service failed to start"
    kill $USER_SERVICE_PID 2>/dev/null || true
    exit 1
fi

echo "All services started successfully!"
echo "User Service PID: $USER_SERVICE_PID (gRPC on port 9001)"
echo "HelloWorld Service PID: $HELLOWORLD_SERVICE_PID (HTTP on port 8080)"
echo ""
echo "Service endpoints:"
echo "  - HelloWorld Service: http://localhost:8080"
echo "  - Health Check: http://localhost:8080/health"
echo "  - Generate ID: http://localhost:8080/id"
echo "  - Hello World: http://localhost:8080/hello"
echo ""
echo "Logs are available in the logs/ directory"
echo "To stop all services, run: ./scripts/stop-all-services.sh"

# 保存 PID 到文件以便后续停止
echo $USER_SERVICE_PID > logs/user-service.pid
echo $HELLOWORLD_SERVICE_PID > logs/helloworld-service.pid

# 等待用户中断
echo "Press Ctrl+C to stop all services..."
trap 'echo "Stopping services..."; kill $USER_SERVICE_PID $HELLOWORLD_SERVICE_PID 2>/dev/null || true; exit 0' INT
wait
