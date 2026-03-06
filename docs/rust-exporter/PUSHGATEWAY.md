# Rust Exporter - Pushgateway 配置指南

## 概述

Rust Exporter 支持两种模式：
1. **Pull 模式**（默认）：Prometheus 主动拉取 `/metrics` 端点
2. **Push 模式**：主动推送指标到 Pushgateway（腾讯云 Prometheus）

两种模式可以同时启用。

---

## 配置 Pushgateway 推送

### 环境变量配置

在 `.env` 文件或环境变量中添加以下配置：

```bash
# 必需：Pushgateway URL
PUSHGATEWAY_URL=https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1-intranet.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway

# 可选：Job 名称（默认：rust-exporter）
PUSHGATEWAY_JOB=channel-health-exporter

# 可选：认证信息（如果需要）
PUSHGATEWAY_USERNAME=your_ak
PUSHGATEWAY_PASSWORD=your_sk

# 可选：推送间隔（秒，默认：60）
PUSHGATEWAY_INTERVAL=60

# 数据库连接（必需）
DATABASE_URL=postgres://user:password@host:5432/database
```

### Docker Compose 配置

更新 `docker-compose.yml`：

```yaml
services:
  rust-exporter:
    build:
      context: ./rust-exporter
      dockerfile: Dockerfile
    container_name: rust-exporter
    environment:
      - DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
      # Pushgateway 配置
      - PUSHGATEWAY_URL=https://your-pushgateway-url/api/v1/pushgateway
      - PUSHGATEWAY_JOB=channel-health-exporter
      - PUSHGATEWAY_USERNAME=${PUSHGATEWAY_AK}
      - PUSHGATEWAY_PASSWORD=${PUSHGATEWAY_SK}
      - PUSHGATEWAY_INTERVAL=60
    ports:
      - "8001:8001"
      - "8002:8002"
    restart: unless-stopped
    networks:
      - monitoring
```

---

## 使用场景

### 场景 1: 只使用腾讯云 Prometheus（推荐）

```bash
# .env 配置
DATABASE_URL=postgres://...
PUSHGATEWAY_URL=https://your-tencent-cloud-pushgateway-url
PUSHGATEWAY_USERNAME=your_ak
PUSHGATEWAY_PASSWORD=your_sk
```

**特点**：
- ✅ 数据推送到腾讯云
- ✅ 本地 `/metrics` 端点仍然可用（用于测试）
- ✅ 无需自建 Prometheus

### 场景 2: 双模式（腾讯云 + 本地）

```bash
# .env 配置
DATABASE_URL=postgres://...
PUSHGATEWAY_URL=https://your-tencent-cloud-pushgateway-url
PUSHGATEWAY_USERNAME=your_ak
PUSHGATEWAY_PASSWORD=your_sk
```

**docker-compose.yml** 保留本地 Prometheus：

```yaml
services:
  rust-exporter:
    # ... 配置 Pushgateway
  
  prometheus:
    # 本地 Prometheus 用于测试和 Grafana
    image: prom/prometheus:latest
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"
```

**特点**：
- ✅ 生产数据推送到腾讯云
- ✅ 本地 Prometheus 用于开发测试
- ✅ Grafana 可以连接本地 Prometheus 查看实时数据

### 场景 3: 只使用本地 Prometheus

```bash
# .env 配置 - 不设置 PUSHGATEWAY_URL
DATABASE_URL=postgres://...
```

**特点**：
- ✅ 完全本地部署
- ✅ 不依赖云服务
- ❌ 需要自己维护 Prometheus

---

## 验证配置

### 1. 检查日志

```bash
# Docker 部署
docker compose logs -f rust-exporter

# 应该看到：
# Starting Rust Exporter...
# Database connection pool created
# Registered 3 collectors
# Pushgateway is enabled, starting worker...
# Starting Pushgateway worker: URL=https://..., Job=channel-health-exporter, Interval=60s
# Starting HTTP server on 0.0.0.0:8001
# Collected metrics from availability
# Successfully pushed metrics to Pushgateway
```

### 2. 测试本地端点

```bash
# 健康检查
curl http://localhost:8001/health

# 查看指标（本地）
curl http://localhost:8001/metrics
```

### 3. 验证腾讯云数据

登录腾讯云 Prometheus 控制台，查询指标：

```promql
channel_availability_percent
channel_avg_cost_cny_all
channel_cache_reuse_percent
```

---

## 推送格式

### 指标格式

推送到 Pushgateway 的数据格式：

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

### URL 格式

```
POST {PUSHGATEWAY_URL}/metrics/job/{JOB_NAME}
Content-Type: text/plain; version=0.0.4; charset=utf-8
Authorization: Basic {base64(username:password)}

{metrics_text}
```

---

## 故障排查

### 问题 1: 推送失败

**症状**：日志显示 "Failed to push metrics to Pushgateway"

**排查**：
```bash
# 检查 URL 是否正确
echo $PUSHGATEWAY_URL

# 测试连接
curl -u "ak:sk" -X POST \
  -H "Content-Type: text/plain; version=0.0.4; charset=utf-8" \
  --data-binary "test_metric 1.0" \
  "$PUSHGATEWAY_URL/metrics/job/test"
```

**常见原因**：
1. URL 格式错误（检查是否包含 `/api/v1/pushgateway`）
2. 认证信息错误（AK/SK）
3. 网络不通（防火墙、VPC）

### 问题 2: 认证失败

**症状**：401 Unauthorized

**解决**：
```bash
# 检查 AK/SK 是否正确
# 确保 RAM 用户有 CMS 读写权限
# AliyunPrometheusMetricWriteAccess 或 AliyunCloudMonitorFullAccess
```

### 问题 3: 数据未显示

**症状**：推送成功但腾讯云查不到数据

**排查**：
1. 检查 Job 名称是否正确
2. 等待 1-2 分钟（数据延迟）
3. 检查指标名称拼写
4. 查看腾讯云 Prometheus 日志

---

## 性能考虑

### 推送间隔

```bash
# 默认 60 秒
PUSHGATEWAY_INTERVAL=60

# 高频推送（30 秒）- 增加成本
PUSHGATEWAY_INTERVAL=30

# 低频推送（120 秒）- 降低成本
PUSHGATEWAY_INTERVAL=120
```

### 网络开销

- 每次推送约 1-2 KB 数据
- 60 秒间隔：~1.4 MB/天
- 30 秒间隔：~2.8 MB/天

### 成本优化

1. **合理设置推送间隔**：根据业务需求选择 60-120 秒
2. **只推送必要指标**：可以在代码中过滤指标
3. **使用内网地址**：如果在腾讯云内部署，使用内网 URL

---

## 完整示例

### .env 文件

```bash
# 数据库
DATABASE_URL=postgres://dev_read_chunqiu:password@host:5432/claude_code

# 腾讯云 Prometheus Pushgateway
PUSHGATEWAY_URL=https://workspace-default-cms-xxx.log.aliyuncs.com/prometheus/workspace-xxx/aliyun-prom-xxx/api/v1/pushgateway
PUSHGATEWAY_JOB=channel-health-exporter
PUSHGATEWAY_USERNAME=your_access_key
PUSHGATEWAY_PASSWORD=your_secret_key
PUSHGATEWAY_INTERVAL=60
```

### 启动命令

```bash
# Docker 部署
docker compose up -d rust-exporter

# 本地开发
cd rust-exporter
source .env
cargo run --release
```

### 验证

```bash
# 1. 检查日志
docker compose logs -f rust-exporter | grep -i pushgateway

# 2. 查看本地指标
curl http://localhost:8001/metrics | grep channel_

# 3. 登录腾讯云控制台验证数据
```

---

## 迁移指南

### 从本地 Prometheus 迁移到腾讯云

1. **添加 Pushgateway 配置**（不删除本地 Prometheus）
2. **观察 1-2 天**，确保数据正常推送
3. **对比数据**，确保腾讯云和本地数据一致
4. **切换 Grafana 数据源**到腾讯云
5. **（可选）停止本地 Prometheus**

### 回滚方案

如果腾讯云出现问题，可以立即切回本地：

```bash
# 1. 停止推送
unset PUSHGATEWAY_URL
docker compose restart rust-exporter

# 2. Grafana 切回本地数据源
# Configuration > Data Sources > 选择本地 Prometheus
```

---

## 总结

**推荐配置**（生产环境）：

```bash
# 主要推送到腾讯云
PUSHGATEWAY_URL=https://your-tencent-cloud-url
PUSHGATEWAY_USERNAME=your_ak
PUSHGATEWAY_PASSWORD=your_sk
PUSHGATEWAY_INTERVAL=60

# 保留本地 Prometheus 作为备份
# docker-compose.yml 中保留 prometheus 服务
```

**优势**：
- ✅ 生产数据在腾讯云（高可用）
- ✅ 本地有备份（应急）
- ✅ 开发测试方便
- ✅ 成本可控

---

**相关文档**：
- [DEPLOYMENT.md](./DEPLOYMENT.md) - 完整部署指南
- [E2E_VALIDATION.md](./E2E_VALIDATION.md) - 端到端验证
