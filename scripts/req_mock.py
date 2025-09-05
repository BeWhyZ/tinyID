#!/usr/bin/env python3
# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "aiohttp",
# ]
# ///
"""
并发请求测试脚本
发送并发请求到 http://localhost:8080/user?id=<random_id>
"""

import asyncio
import aiohttp
import random
import time
import argparse
from typing import Dict, List
import json


class RequestStats:
    """请求统计类"""
    def __init__(self):
        self.total_requests = 0
        self.successful_requests = 0
        self.failed_requests = 0
        self.response_times = []
        self.errors = []
    
    def add_success(self, response_time: float):
        self.total_requests += 1
        self.successful_requests += 1
        self.response_times.append(response_time)
    
    def add_failure(self, error: str, response_time: float = 0):
        self.total_requests += 1
        self.failed_requests += 1
        self.errors.append(error)
        if response_time > 0:
            self.response_times.append(response_time)
    
    def get_summary(self) -> Dict:
        """获取统计摘要"""
        if self.response_times:
            avg_response_time = sum(self.response_times) / len(self.response_times)
            min_response_time = min(self.response_times)
            max_response_time = max(self.response_times)
        else:
            avg_response_time = min_response_time = max_response_time = 0
        
        return {
            "total_requests": self.total_requests,
            "successful_requests": self.successful_requests,
            "failed_requests": self.failed_requests,
            "success_rate": (self.successful_requests / self.total_requests * 100) if self.total_requests > 0 else 0,
            "avg_response_time_ms": round(avg_response_time * 1000, 2),
            "min_response_time_ms": round(min_response_time * 1000, 2),
            "max_response_time_ms": round(max_response_time * 1000, 2),
            "errors": self.errors[:10]  # 只显示前10个错误
        }


async def make_request(session: aiohttp.ClientSession, base_url: str, stats: RequestStats, 
                      min_id: int = 1, max_id: int = 10000) -> None:
    """发送单个请求"""
    user_id = random.randint(min_id, max_id)
    url = f"{base_url}/user?id={user_id}"
    
    start_time = time.time()
    try:
        async with session.get(url) as response:
            end_time = time.time()
            response_time = end_time - start_time
            
            # 读取响应内容
            content = await response.text()
            
            if response.status == 200:
                stats.add_success(response_time)
                print(f"✓ ID:{user_id} Status:{response.status} Time:{response_time*1000:.2f}ms")
            else:
                error_msg = f"HTTP {response.status}: {content[:100]}"
                stats.add_failure(error_msg, response_time)
                print(f"✗ ID:{user_id} Status:{response.status} Time:{response_time*1000:.2f}ms Error:{content[:50]}")
                
    except aiohttp.ClientError as e:
        end_time = time.time()
        response_time = end_time - start_time
        error_msg = f"ClientError: {str(e)}"
        stats.add_failure(error_msg, response_time)
        print(f"✗ ID:{user_id} ClientError: {e}")
    except Exception as e:
        end_time = time.time()
        response_time = end_time - start_time
        error_msg = f"Exception: {str(e)}"
        stats.add_failure(error_msg, response_time)
        print(f"✗ ID:{user_id} Exception: {e}")


async def run_concurrent_requests(base_url: str, num_requests: int, concurrency: int,
                                 min_id: int = 1, max_id: int = 10000) -> RequestStats:
    """运行并发请求"""
    stats = RequestStats()
    
    # 创建连接池
    connector = aiohttp.TCPConnector(limit=concurrency, limit_per_host=concurrency)
    timeout = aiohttp.ClientTimeout(total=30)  # 30秒超时
    
    async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
        # 创建信号量来控制并发数
        semaphore = asyncio.Semaphore(concurrency)
        
        async def bounded_request():
            async with semaphore:
                await make_request(session, base_url, stats, min_id, max_id)
        
        # 创建所有任务
        tasks = [bounded_request() for _ in range(num_requests)]
        
        print(f"开始发送 {num_requests} 个并发请求 (并发度: {concurrency})...")
        print(f"目标URL: {base_url}/user?id=<{min_id}-{max_id}>")
        print("-" * 80)
        
        start_time = time.time()
        
        # 执行所有任务
        await asyncio.gather(*tasks, return_exceptions=True)
        
        end_time = time.time()
        total_time = end_time - start_time
        
        print("-" * 80)
        print(f"所有请求完成，总耗时: {total_time:.2f}s")
        
        # 计算QPS
        qps = num_requests / total_time if total_time > 0 else 0
        print(f"QPS (每秒请求数): {qps:.2f}")
    
    return stats


def print_statistics(stats: RequestStats):
    """打印统计信息"""
    summary = stats.get_summary()
    
    print("\n" + "=" * 60)
    print("📊 请求统计报告")
    print("=" * 60)
    print(f"总请求数:     {summary['total_requests']}")
    print(f"成功请求数:   {summary['successful_requests']}")
    print(f"失败请求数:   {summary['failed_requests']}")
    print(f"成功率:       {summary['success_rate']:.2f}%")
    print(f"平均响应时间: {summary['avg_response_time_ms']:.2f}ms")
    print(f"最小响应时间: {summary['min_response_time_ms']:.2f}ms")
    print(f"最大响应时间: {summary['max_response_time_ms']:.2f}ms")
    
    if summary['errors']:
        print(f"\n❌ 错误信息 (显示前10个):")
        for i, error in enumerate(summary['errors'], 1):
            print(f"  {i}. {error}")
    
    print("=" * 60)


async def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="并发请求测试工具")
    parser.add_argument("--url", default="http://localhost:8080", 
                       help="基础URL (默认: http://localhost:8080)")
    parser.add_argument("-n", "--requests", type=int, default=200,
                       help="总请求数 (默认: 100)")
    parser.add_argument("-c", "--concurrency", type=int, default=100,
                       help="并发数 (默认: 10)")
    parser.add_argument("--min-id", type=int, default=1,
                       help="用户ID最小值 (默认: 1)")
    parser.add_argument("--max-id", type=int, default=10000,
                       help="用户ID最大值 (默认: 10000)")
    parser.add_argument("--json-output", action="store_true",
                       help="以JSON格式输出结果")
    
    args = parser.parse_args()
    
    # 验证参数
    if args.requests <= 0:
        print("❌ 请求数必须大于0")
        return
    
    if args.concurrency <= 0:
        print("❌ 并发数必须大于0")
        return
    
    if args.min_id >= args.max_id:
        print("❌ 最小ID必须小于最大ID")
        return
    
    try:
        # 运行并发请求
        stats = await run_concurrent_requests(
            args.url, args.requests, args.concurrency, args.min_id, args.max_id
        )
        
        if args.json_output:
            # JSON格式输出
            result = stats.get_summary()
            print(json.dumps(result, indent=2, ensure_ascii=False))
        else:
            # 普通格式输出
            print_statistics(stats)
            
    except KeyboardInterrupt:
        print("\n⚠️  用户中断请求")
    except Exception as e:
        print(f"❌ 发生错误: {e}")


if __name__ == "__main__":
    # 设置事件循环策略 (Windows兼容性)
    try:
        if hasattr(asyncio, 'WindowsProactorEventLoopPolicy'):
            asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    except:
        pass
    
    asyncio.run(main())
