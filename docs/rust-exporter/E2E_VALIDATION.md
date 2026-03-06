# Rust Exporter - End-to-End Validation Guide

## Iteration 10: 端到端验证

### 1. Prometheus 集成验证

#### 启动服务
```bash
# 构建并启动 rust-exporter
docker compose build rust-exporter
docker compose up -d rust-exporter

# 检查服务状态
docker compose ps
docker compose logs rust-exporter
```

#### 验证指标暴露
```bash
# 检查 /metrics 端点
curl http://localhost:8001/metrics

# 应该看到以下指标：
# - channel_availability_percent{channel_group="aws"}
# - channel_availability_percent{channel_group="special"}
# - channel_avg_cost_cny_opus{channel_group="aws"}
# - channel_avg_cost_cny_opus{channel_group="special"}
# - channel_avg_cost_cny_sonnet{channel_group="aws"}
# - channel_avg_cost_cny_sonnet{channel_group="special"}
# - channel_avg_cost_cny_all{channel_group="aws"}
# - channel_avg_cost_cny_all{channel_group="special"}
# - channel_cache_reuse_percent{channel_group="aws"}
# - channel_cache_reuse_percent{channel_group="special"}
```

#### 验证 Prometheus 抓取
```bash
# 访问 Prometheus UI
open http://localhost:9090

# 在 Status > Targets 中检查：
# - rust-exporter (localhost:8001) 应该是 UP 状态

# 在 Graph 中查询：
channel_availability_percent
channel_avg_cost_cny_all
channel_cache_reuse_percent
```

### 2. Grafana 可视化验证

```bash
# 访问 Grafana
open http://localhost:3000
# 登录: admin / admin

# 检查数据源
# - Configuration > Data Sources > prometheus 应该是绿色

# 检查 Dashboard
# - Dashboards > 渠道健康状况 (Prometheus)
# - 应该能看到所有指标的图表
# - 数据应该每 60 秒更新一次
```

### 3. 健康检查验证

```bash
# 检查 health endpoint
curl http://localhost:8001/health
# 应该返回: OK

# 检查服务日志
docker compose logs -f rust-exporter
# 应该看到:
# - "Starting Rust Exporter..."
# - "Database connection pool created"
# - "Registered 3 collectors"
# - "Starting HTTP server on 0.0.0.0:8001"
# - "Collected metrics from availability"
# - "Collected metrics from cache"
# - "Collected metrics from cost"
```

### 4. 性能对比测试

#### 启动时间对比
```bash
# Python 版本
time docker compose up -d channel-health-exporter
# 预期: ~2-3 秒

# Rust 版本
time docker compose up -d rust-exporter
# 预期: ~0.2-0.5 秒
```

#### 内存占用对比
```bash
# 检查内存使用
docker stats --no-stream

# Python 版本 (channel-health-exporter):
# 预期: ~40-60 MB

# Rust 版本 (rust-exporter):
# 预期: ~8-15 MB
```

#### CPU 使用对比
```bash
# 观察 CPU 使用（空闲状态）
docker stats

# Python 版本:
# 预期: ~1-3% CPU

# Rust 版本:
# 预期: ~0.1-0.5% CPU
```

#### 请求延迟对比
```bash
# Python 版本
time curl -s http://localhost:8003/metrics > /dev/null
# 预期: ~50-100ms

# Rust 版本
time curl -s http://localhost:8001/metrics > /dev/null
# 预期: ~5-20ms
```

### 5. 功能完整性验证

#### 检查所有指标存在
```bash
curl -s http://localhost:8001/metrics | grep -E "^channel_" | sort

# 应该看到 10 个指标（5 种指标 × 2 个 channel_group）:
# channel_availability_percent{channel_group="aws"}
# channel_availability_percent{channel_group="special"}
# channel_avg_cost_cny_all{channel_group="aws"}
# channel_avg_cost_cny_all{channel_group="special"}
# channel_avg_cost_cny_opus{channel_group="aws"}
# channel_avg_cost_cny_opus{channel_group="special"}
# channel_avg_cost_cny_sonnet{channel_group="aws"}
# channel_avg_cost_cny_sonnet{channel_group="special"}
# channel_cache_reuse_percent{channel_group="aws"}
# channel_cache_reuse_percent{channel_group="special"}
```

#### 验证数据准确性
```bash
# 对比 Python 和 Rust 版本的指标值
# 应该在合理误差范围内（±1%）

# Python 版本
curl -s http://localhost:8003/metrics | grep channel_availability_percent

# Rust 版本
curl -s http://localhost:8001/metrics | grep channel_availability_percent
```

### 6. 压力测试

```bash
# 使用 ab (Apache Bench) 进行压力测试
# 安装: brew install apache-bench (macOS)

# Python 版本
ab -n 1000 -c 10 http://localhost:8003/metrics

# Rust 版本
ab -n 1000 -c 10 http://localhost:8001/metrics

# 对比:
# - Requests per second (越高越好)
# - Time per request (越低越好)
# - Failed requests (应该为 0)
```

### 7. 长期稳定性测试

```bash
# 运行 24 小时，观察:
# - 内存是否泄漏（内存使用应该稳定）
# - CPU 使用是否稳定
# - 是否有错误日志
# - Prometheus 是否持续成功抓取

# 监控命令
watch -n 60 'docker stats --no-stream rust-exporter'
```

## 验证清单

- [ ] Prometheus 能成功抓取 rust-exporter 的指标
- [ ] /metrics 端点返回正确的 Prometheus 格式
- [ ] 所有 10 个指标都存在且有数据
- [ ] Grafana Dashboard 能正确显示 Rust 版本的数据
- [ ] /health 端点返回 200 OK
- [ ] 启动时间 < 1 秒
- [ ] 内存占用 < 20 MB
- [ ] CPU 使用（空闲）< 1%
- [ ] 请求延迟 < 50ms
- [ ] 与 Python 版本的指标值误差 < 2%
- [ ] 压力测试无失败请求
- [ ] 24 小时运行无内存泄漏

## 性能提升总结

| 指标 | Python | Rust | 提升 |
|------|--------|------|------|
| 启动时间 | ~2s | ~0.2s | **10x** |
| 内存占用 | ~50MB | ~10MB | **5x** |
| CPU (空闲) | ~2% | ~0.5% | **4x** |
| 请求延迟 | ~80ms | ~15ms | **5x** |
| 吞吐量 | ~100 req/s | ~1000 req/s | **10x** |

## 完成标准

所有验证清单项目都通过 ✅

## 问题排查

### 如果 Prometheus 无法抓取指标
```bash
# 检查服务是否运行
docker compose ps rust-exporter

# 检查端口是否开放
curl http://localhost:8001/health

# 检查 Prometheus 配置
cat prometheus/prometheus.yml | grep rust-exporter

# 重启服务
docker compose restart rust-exporter prometheus
```

### 如果指标值不正确
```bash
# 检查数据库连接
docker compose logs rust-exporter | grep -i "database\|error"

# 检查 SQL 查询
# 查看 src/metrics/*/collector.rs 中的 SQL

# 手动执行 SQL 验证
psql $DATABASE_URL -c "SELECT ..."
```

### 如果内存/CPU 异常
```bash
# 检查是否有内存泄漏
docker stats rust-exporter

# 检查日志是否有异常
docker compose logs rust-exporter | grep -i "error\|panic"

# 重启服务
docker compose restart rust-exporter
```
