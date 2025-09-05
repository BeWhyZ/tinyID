#!/bin/bash

# 停止所有微服务的脚本

set -e

echo "Stopping all microservices..."

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the project root directory"
    exit 1
fi

# 从 PID 文件读取进程ID并停止服务
if [ -f "logs/user-service.pid" ]; then
    USER_SERVICE_PID=$(cat logs/user-service.pid)
    if kill -0 $USER_SERVICE_PID 2>/dev/null; then
        echo "Stopping User Service (PID: $USER_SERVICE_PID)..."
        kill $USER_SERVICE_PID
        sleep 2
        # 如果进程仍在运行，强制终止
        if kill -0 $USER_SERVICE_PID 2>/dev/null; then
            echo "Force killing User Service..."
            kill -9 $USER_SERVICE_PID 2>/dev/null || true
        fi
    else
        echo "User Service is not running"
    fi
    rm -f logs/user-service.pid
else
    echo "No User Service PID file found"
fi

if [ -f "logs/helloworld-service.pid" ]; then
    HELLOWORLD_SERVICE_PID=$(cat logs/helloworld-service.pid)
    if kill -0 $HELLOWORLD_SERVICE_PID 2>/dev/null; then
        echo "Stopping HelloWorld Service (PID: $HELLOWORLD_SERVICE_PID)..."
        kill $HELLOWORLD_SERVICE_PID
        sleep 2
        # 如果进程仍在运行，强制终止
        if kill -0 $HELLOWORLD_SERVICE_PID 2>/dev/null; then
            echo "Force killing HelloWorld Service..."
            kill -9 $HELLOWORLD_SERVICE_PID 2>/dev/null || true
        fi
    else
        echo "HelloWorld Service is not running"
    fi
    rm -f logs/helloworld-service.pid
else
    echo "No HelloWorld Service PID file found"
fi

# 清理可能残留的进程
echo "Cleaning up any remaining processes..."
pkill -f "user-service" 2>/dev/null || true
pkill -f "helloworld-service" 2>/dev/null || true

echo "All services stopped successfully!"
