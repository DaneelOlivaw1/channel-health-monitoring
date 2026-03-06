#!/bin/bash

echo "=== Rust Exporter 测试脚本 ==="
echo ""
echo "⚠️  注意：第一次启动需要等待 2-3 分钟才会有数据"
echo ""

# 检查是否刚启动
UPTIME=$(docker compose ps rust-exporter --format json 2>/dev/null | grep -o '"Status":"[^"]*"' | cut -d'"' -f4)
echo "服务状态: $UPTIME"
echo ""

# 如果刚启动，等待
if [[ "$UPTIME" == *"second"* ]] || [[ "$UPTIME" == *"minute"* ]]; then
    SECONDS=$(echo "$UPTIME" | grep -o '[0-9]*' | head -1)
    if [ "$SECONDS" -lt 120 ]; then
        WAIT_TIME=$((120 - SECONDS))
        echo "服务刚启动，等待 $WAIT_TIME 秒让采集器运行..."
        sleep $WAIT_TIME
        echo ""
    fi
fi

# 1. 检查服务状态
echo "1. 检查服务状态..."
docker compose ps | grep rust-exporter
echo ""

# 2. 健康检查
echo "2. 健康检查..."
HEALTH=$(curl -s http://localhost:8001/health)
if [ "$HEALTH" == "OK" ]; then
    echo "✅ 健康检查通过: $HEALTH"
else
    echo "❌ 健康检查失败: $HEALTH"
fi
echo ""

# 3. 检查指标数量
echo "3. 检查指标数量..."
METRIC_COUNT=$(curl -s http://localhost:8001/metrics | grep -c "^channel_.*{")
echo "找到 $METRIC_COUNT 个指标数据点"
if [ "$METRIC_COUNT" -eq 0 ]; then
    echo "⚠️  指标为空！可能原因："
    echo "   - 数据库最近 3 小时没有数据"
    echo "   - 采集器还没运行（等待 60 秒）"
    echo "   - 数据库连接失败"
    echo ""
    echo "检查最近的日志："
    docker compose logs rust-exporter | grep -E "(Collected|Failed)" | tail -5
else
    echo "✅ 指标正常"
fi
echo ""

# 4. 检查每个指标
echo "4. 检查各个指标..."
echo "- Availability:"
AVAIL=$(curl -s http://localhost:8001/metrics | grep "channel_availability_percent{" | head -2)
if [ -z "$AVAIL" ]; then
    echo "  ⚠️  无数据"
else
    echo "$AVAIL"
fi
echo ""

echo "- Cost (Opus):"
OPUS=$(curl -s http://localhost:8001/metrics | grep "channel_avg_cost_cny_opus{" | head -2)
if [ -z "$OPUS" ]; then
    echo "  ⚠️  无数据"
else
    echo "$OPUS"
fi
echo ""

echo "- Cache:"
CACHE=$(curl -s http://localhost:8001/metrics | grep "channel_cache_reuse_percent{" | head -2)
if [ -z "$CACHE" ]; then
    echo "  ⚠️  无数据"
else
    echo "$CACHE"
fi
echo ""

# 5. 检查 Pushgateway 状态
echo "5. 检查 Pushgateway 状态..."
PUSH_STATUS=$(docker compose logs rust-exporter 2>/dev/null | grep -i "pushgateway" | tail -1)
if [[ "$PUSH_STATUS" == *"disabled"* ]]; then
    echo "ℹ️  Pushgateway 未启用（仅本地模式）"
elif [[ "$PUSH_STATUS" == *"enabled"* ]]; then
    echo "✅ Pushgateway 已启用"
    docker compose logs rust-exporter | grep -i "pushgateway" | tail -3
else
    echo "⚠️  无法确定 Pushgateway 状态"
fi
echo ""

# 6. 检查最近的采集日志
echo "6. 最近的采集日志..."
docker compose logs rust-exporter 2>/dev/null | grep "Collected metrics" | tail -5
echo ""

# 7. 数据库检查（如果指标为空）
if [ "$METRIC_COUNT" -eq 0 ]; then
    echo "7. 检查数据库数据..."
    echo "⚠️  指标为空，建议检查数据库是否有最近 3 小时的数据"
    echo ""
    echo "运行以下命令检查："
    echo 'psql "postgres://dev_read_chunqiu:PASSWORD@HOST:5432/claude_code" -c "'
    echo "SELECT COUNT(*) as total, MAX(created_at) as latest"
    echo "FROM channel_request_log"
    echo "WHERE created_at >= NOW() - INTERVAL '3 hours';"
    echo '"'
fi

echo ""
echo "=== 测试完成 ==="
echo ""
echo "📝 说明："
echo "- 如果指标为空，这是正常的（数据需要累积）"
echo "- 等待 2-3 分钟后重新运行此脚本"
echo "- 确保数据库有最近 3 小时的数据"
