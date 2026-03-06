# Rust Exporter 本地测试指南

## 测试场景

### 场景 1: 只测试本地功能（不推送到阿里云）

这是最简单的测试方式，不需要配置阿里云。

#### 步骤 1: 配置环境变量

```bash
cd /Users/daneel/project/proxy_project-observatory
cp .env.example .env
vim .env
```

**只配置数据库，不配置 Pushgateway**：

```bash
# 数据库连接
DB_HOST=pgm-6we2iwj50gul9ez8eo.pgsql.japan.rds.aliyuncs.com
DB_PORT=5432
DB_NAME=claude_code
DB_USER=dev_read_chunqiu
DB_PASSWORD=w7QcN8zp2VxT5Rb

# Pushgateway 留空（不推送到阿里云）
PUSHGATEWAY_URL=
PUSHGATEWAY_USERNAME=
PUSHGATEWAY_PASSWORD=
```

#### 步骤 2: 启动服务

```bash
# 启动 Rust Exporter + 本地 Prometheus + Grafana
docker compose up -d

# 查看日志
docker compose logs -f rust-exporter
```

**预期日志**：
```
Starting Rust Exporter...
Database connection pool created
Registered 3 collectors
Pushgateway is disabled  ← 注意这里
Starting HTTP server on 0.0.0.0:8001
Collected metrics from availability
Collected metrics from cache
Collected metrics from cost
```

#### 步骤 3: 测试本地端点

```bash
# 1. 健康检查
curl http://localhost:8001/health
# 应该返回: OK

# 2. 查看所有指标
curl http://localhost:8001/metrics

# 3. 查看特定指标
curl http://localhost:8001/metrics | grep channel_availability_percent
curl http://localhost:8001/metrics | grep channel_avg_cost_cny
curl http://localhost:8001/metrics | grep channel_cache_reuse_percent
```

**预期输出**：
```
# HELP channel_availability_percent Channel availability percentage
# TYPE channel_availability_percent gauge
channel_availability_percent{channel_group="aws"} 95.5
channel_availability_percent{channel_group="special"} 98.2

# HELP channel_avg_cost_cny_opus Average cost for Opus model
# TYPE channel_avg_cost_cny_opus gauge
channel_avg_cost_cny_opus{channel_group="aws"} 0.54
channel_avg_cost_cny_opus{channel_group="special"} 0.60

# HELP channel_cache_reuse_percent Cache reuse percentage
# TYPE channel_cache_reuse_percent gauge
channel_cache_reuse_percent{channel_group="aws"} 85.5
channel_cache_reuse_percent{channel_group="special"} 92.3
```

#### 步骤 4: 验证 Prometheus 抓取

```bash
# 访问 Prometheus UI
open http://localhost:9090

# 或使用 curl 查询
curl 'http://localhost:9090/api/v1/query?query=channel_availability_percent'
```

在 Prometheus UI 中：
1. 点击 **Status > Targets**
2. 应该看到 `rust-exporter (localhost:8001)` 状态为 **UP**
3. 点击 **Graph**
4. 输入查询：`channel_availability_percent`
5. 点击 **Execute**，应该看到数据

#### 步骤 5: 验证 Grafana

```bash
# 访问 Grafana
open http://localhost:3000
# 登录: admin / admin
```

1. 左侧导航 > **Explore**
2. 数据源选择 **prometheus**
3. 输入查询：`channel_availability_percent`
4. 点击 **Run query**
5. 应该看到图表显示数据

---

## 场景 2: 测试推送到阿里云（完整测试）

#### 步骤 1: 配置完整环境变量

```bash
vim .env
```

```bash
# 数据库连接
DB_HOST=pgm-6we2iwj50gul9ez8eo.pgsql.japan.rds.aliyuncs.com
DB_PORT=5432
DB_NAME=claude_code
DB_USER=dev_read_chunqiu
DB_PASSWORD=w7QcN8zp2VxT5Rb

# 阿里云 Prometheus Pushgateway
PUSHGATEWAY_URL=https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway
PUSHGATEWAY_JOB=channel-health-exporter
PUSHGATEWAY_USERNAME=你的AccessKey_ID
PUSHGATEWAY_PASSWORD=你的AccessKey_Secret
PUSHGATEWAY_INTERVAL=60
```

#### 步骤 2: 启动服务

```bash
docker compose up -d rust-exporter

# 查看日志
docker compose logs -f rust-exporter
```

**预期日志**：
```
Starting Rust Exporter...
Database connection pool created
Registered 3 collectors
Pushgateway is enabled, starting worker...  ← 注意这里
Starting Pushgateway worker: URL=https://workspace-default-cms-..., Job=channel-health-exporter, Interval=60s
Starting HTTP server on 0.0.0.0:8001
Collected metrics from availability
Successfully pushed metrics to Pushgateway  ← 推送成功
```

#### 步骤 3: 测试本地端点（同场景 1）

```bash
curl http://localhost:8001/health
curl http://localhost:8001/metrics | grep channel_
```

#### 步骤 4: 验证阿里云数据

1. 登录 https://cloudmonitor.console.aliyuncs.com/
2. 左侧导航：**Prometheus 监控 > 实例列表**
3. 进入你的实例
4. 点击 **大盘列表**
5. 选择 **Explore**
6. 输入查询：`channel_availability_percent`
7. 应该看到数据（等待 1-2 分钟）

---

## 场景 3: 本地开发测试（不用 Docker）

适合快速开发和调试。

#### 步骤 1: 安装 Rust

```bash
# 如果还没有安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 步骤 2: 配置环境变量

```bash
cd /Users/daneel/project/proxy_project-observatory/rust-exporter

# 创建 .env 文件
cat > .env << 'EOF'
DATABASE_URL=postgres://dev_read_chunqiu:w7QcN8zp2VxT5Rb@pgm-6we2iwj50gul9ez8eo.pgsql.japan.rds.aliyuncs.com:5432/claude_code

# 可选：推送到阿里云
# PUSHGATEWAY_URL=https://workspace-default-cms-...
# PUSHGATEWAY_USERNAME=your_ak
# PUSHGATEWAY_PASSWORD=your_sk
EOF
```

#### 步骤 3: 运行

```bash
# 开发模式（带调试信息）
cargo run

# 或发布模式（优化性能）
cargo run --release
```

#### 步骤 4: 测试

```bash
# 另开一个终端
curl http://localhost:8001/health
curl http://localhost:8001/metrics | grep channel_
```

#### 步骤 5: 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test availability_collector_tests

# 查看测试输出
cargo test -- --nocapture
```

---

## 测试检查清单

### ✅ 基础功能测试

```bash
# 1. 服务启动
docker compose ps | grep rust-exporter
# 应该显示 "Up"

# 2. 健康检查
curl http://localhost:8001/health
# 应该返回 "OK"

# 3. 指标端点
curl http://localhost:8001/metrics | head -20
# 应该看到 Prometheus 格式的指标

# 4. 检查所有指标存在
curl -s http://localhost:8001/metrics | grep -E "^channel_" | wc -l
# 应该返回 10（5 种指标 × 2 个 channel_group）

# 5. 检查指标值
curl -s http://localhost:8001/metrics | grep channel_availability_percent
curl -s http://localhost:8001/metrics | grep channel_avg_cost_cny_opus
curl -s http://localhost:8001/metrics | grep channel_avg_cost_cny_sonnet
curl -s http://localhost:8001/metrics | grep channel_avg_cost_cny_all
curl -s http://localhost:8001/metrics | grep channel_cache_reuse_percent
```

### ✅ Prometheus 集成测试

```bash
# 1. 检查 Prometheus 配置
cat prometheus/prometheus.yml | grep rust-exporter

# 2. 检查 Prometheus Targets
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.labels.job=="rust-exporter")'

# 3. 查询指标
curl 'http://localhost:9090/api/v1/query?query=channel_availability_percent' | jq .

# 4. 查询时间序列
curl 'http://localhost:9090/api/v1/query_range?query=channel_availability_percent&start=2026-03-06T00:00:00Z&end=2026-03-06T23:59:59Z&step=60s' | jq .
```

### ✅ Pushgateway 测试（如果配置了）

```bash
# 1. 检查日志
docker compose logs rust-exporter | grep -i pushgateway

# 2. 应该看到
# - "Pushgateway is enabled"
# - "Starting Pushgateway worker"
# - "Successfully pushed metrics to Pushgateway"

# 3. 手动测试推送
curl -u 'your_ak:your_sk' \
  -H "Content-Type: text/plain; version=0.0.4; charset=utf-8" \
  --data-binary "test_metric 1.0" \
  "https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway/metrics/job/test"
```

---

## 常见测试问题

### 问题 1: 指标值为 0 或空

**原因**：数据库中没有数据或查询条件不匹配

**排查**：
```bash
# 检查数据库连接
docker compose logs rust-exporter | grep -i "database\|error"

# 手动查询数据库
psql "postgres://dev_read_chunqiu:w7QcN8zp2VxT5Rb@pgm-6we2iwj50gul9ez8eo.pgsql.japan.rds.aliyuncs.com:5432/claude_code" -c "
SELECT 
    CASE WHEN channel_code='aws' THEN 'aws' ELSE 'special' END as grp,
    COUNT(*) as total
FROM channel_request_log
WHERE created_at >= NOW() - INTERVAL '3 hours'
    AND channel_code IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
GROUP BY grp;
"
```

### 问题 2: Prometheus 无法抓取

**原因**：网络或配置问题

**排查**：
```bash
# 1. 检查端口
lsof -i :8001

# 2. 测试连接
curl http://localhost:8001/metrics

# 3. 检查 Prometheus 配置
docker compose exec prometheus cat /etc/prometheus/prometheus.yml | grep rust-exporter

# 4. 重启 Prometheus
docker compose restart prometheus
```

### 问题 3: 推送到阿里云失败

**原因**：认证或网络问题

**排查**：
```bash
# 1. 检查 AK/SK
echo $PUSHGATEWAY_USERNAME
echo $PUSHGATEWAY_PASSWORD

# 2. 测试网络连通性
curl -I https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1.log.aliyuncs.com

# 3. 测试认证
curl -u 'your_ak:your_sk' \
  -H "Content-Type: text/plain; version=0.0.4; charset=utf-8" \
  --data-binary "test 1" \
  "https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway/metrics/job/test"
```

---

## 性能测试

### 测试启动时间

```bash
time docker compose up -d rust-exporter
# Rust 版本应该 < 1 秒
```

### 测试内存占用

```bash
docker stats rust-exporter --no-stream
# 应该 < 20 MB
```

### 测试请求延迟

```bash
# 测试 100 次请求
for i in {1..100}; do
  time curl -s http://localhost:8001/metrics > /dev/null
done

# 或使用 ab (Apache Bench)
ab -n 1000 -c 10 http://localhost:8001/metrics
```

---

## 快速测试脚本

创建一个测试脚本：

```bash
cat > test-rust-exporter.sh << 'EOF'
#!/bin/bash

echo "=== Rust Exporter 测试脚本 ==="
echo ""

# 1. 检查服务状态
echo "1. 检查服务状态..."
docker compose ps | grep rust-exporter
echo ""

# 2. 健康检查
echo "2. 健康检查..."
curl -s http://localhost:8001/health
echo ""
echo ""

# 3. 检查指标数量
echo "3. 检查指标数量..."
METRIC_COUNT=$(curl -s http://localhost:8001/metrics | grep -c "^channel_")
echo "找到 $METRIC_COUNT 个指标（预期：10）"
echo ""

# 4. 检查每个指标
echo "4. 检查各个指标..."
echo "- Availability:"
curl -s http://localhost:8001/metrics | grep channel_availability_percent
echo ""
echo "- Cost (Opus):"
curl -s http://localhost:8001/metrics | grep channel_avg_cost_cny_opus
echo ""
echo "- Cost (Sonnet):"
curl -s http://localhost:8001/metrics | grep channel_avg_cost_cny_sonnet
echo ""
echo "- Cost (All):"
curl -s http://localhost:8001/metrics | grep channel_avg_cost_cny_all
echo ""
echo "- Cache:"
curl -s http://localhost:8001/metrics | grep channel_cache_reuse_percent
echo ""

# 5. 检查 Pushgateway 状态
echo "5. 检查 Pushgateway 状态..."
docker compose logs rust-exporter | grep -i pushgateway | tail -5
echo ""

# 6. 检查最近的采集日志
echo "6. 最近的采集日志..."
docker compose logs rust-exporter | grep "Collected metrics" | tail -5
echo ""

echo "=== 测试完成 ==="
EOF

chmod +x test-rust-exporter.sh
./test-rust-exporter.sh
```

---

## 总结

**推荐测试流程**：

1. **本地测试**（场景 1）：先不配置 Pushgateway，验证基本功能
2. **Prometheus 验证**：确认本地 Prometheus 能抓取数据
3. **Grafana 验证**：确认 Grafana 能显示图表
4. **阿里云测试**（场景 2）：配置 Pushgateway，验证推送功能
5. **性能测试**：验证内存、CPU、延迟

**测试通过标准**：
- ✅ 健康检查返回 OK
- ✅ 10 个指标都有数据
- ✅ Prometheus 能抓取数据
- ✅ Grafana 能显示图表
- ✅ （可选）阿里云能查询到数据
- ✅ 内存 < 20MB，CPU < 1%

需要我帮你运行测试或解决任何问题吗？
