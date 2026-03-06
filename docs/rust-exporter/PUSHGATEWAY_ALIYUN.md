# Rust Exporter - 阿里云 Prometheus Pushgateway 配置指南

## 概述

Rust Exporter 支持推送指标到阿里云 Prometheus（通过 Pushgateway），同时保留本地 `/metrics` 端点用于测试。

---

## 快速配置

### 1. 获取阿里云 Pushgateway 地址

根据你的截图，你的 Pushgateway 地址是：

**内网地址**（推荐，如果部署在阿里云内）：
```
https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1-intranet.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway
```

**公网地址**：
```
https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1-log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway
```

### 2. 配置环境变量

创建或编辑 `.env` 文件：

```bash
# 数据库连接
DATABASE_URL=postgres://dev_read_chunqiu:w7QcN8zp2VxT5Rb@pgm-6we2iwj50gul9ez8eo.pgsql.japan.rds.aliyuncs.com:5432/claude_code

# 阿里云 Prometheus Pushgateway 配置
# 使用内网地址（如果在阿里云内部署）
PUSHGATEWAY_URL=https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1-intranet.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway

# Job 名称
PUSHGATEWAY_JOB=channel-health-exporter

# 阿里云 AccessKey（需要有 CMS 读写权限）
PUSHGATEWAY_USERNAME=your_access_key_id
PUSHGATEWAY_PASSWORD=your_access_key_secret

# 推送间隔（秒）
PUSHGATEWAY_INTERVAL=60
```

### 3. 启动服务

```bash
# Docker 部署
docker compose build rust-exporter
docker compose up -d rust-exporter

# 查看日志
docker compose logs -f rust-exporter
```

---

## 阿里云 RAM 权限配置

### 为 RAM 用户授权

如果使用 RAM 用户的 AccessKey，需要授予以下权限之一：

1. **AliyunPrometheusMetricWriteAccess**（推荐，最小权限）
2. **AliyunCloudMonitorFullAccess**（完整权限）

**授权步骤**：
1. 登录 [RAM 控制台](https://ram.console.aliyun.com/)
2. 左侧导航栏选择 **权限管理 > 授权**
3. 点击 **新增授权**
4. 选择 RAM 用户
5. 添加权限策略：`AliyunPrometheusMetricWriteAccess`
6. 确认授权

---

## 验证配置

### 1. 检查日志

```bash
docker compose logs -f rust-exporter
```

**预期输出**：
```
Starting Rust Exporter...
Database connection pool created
Registered 3 collectors
Pushgateway is enabled, starting worker...
Starting Pushgateway worker: URL=https://..., Job=channel-health-exporter, Interval=60s
Starting HTTP server on 0.0.0.0:8001
Collected metrics from availability
Collected metrics from cache
Collected metrics from cost
Successfully pushed metrics to Pushgateway
```

### 2. 测试本地端点

```bash
# 健康检查
curl http://localhost:8001/health

# 查看指标
curl http://localhost:8001/metrics | grep channel_
```

### 3. 验证阿里云数据

1. 登录 [阿里云监控控制台](https://cloudmonitor.console.aliyun.com/)
2. 左侧导航栏选择 **Prometheus 监控 > 实例列表**
3. 进入你的 Prometheus 实例
4. 点击 **大盘列表**，选择 **Explore**
5. 查询指标：

```promql
channel_availability_percent
channel_avg_cost_cny_all
channel_cache_reuse_percent
```

---

## 推送数据格式

### URL 格式

```
POST {PUSHGATEWAY_URL}/metrics/job/{JOB_NAME}/label_key_1/label_value_1
Authorization: Basic {base64(ak:sk)}
Content-Type: text/plain; version=0.0.4; charset=utf-8

{metrics_text}
```

### 示例

```bash
# 使用 curl 测试推送
echo "test_metric 3.14" | curl -u 'your_ak:your_sk' \
  -H "Content-Type: text/plain; version=0.0.4; charset=utf-8" \
  --data-binary @- \
  "https://workspace-default-cms-xxx.log.aliyuncs.com/prometheus/workspace-xxx/aliyun-prom-xxx/api/v1/pushgateway/metrics/job/test_job"
```

### 指标格式

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

---

## Docker Compose 配置

更新 `docker-compose.yml`：

```yaml
services:
  rust-exporter:
    build:
      context: ./rust-exporter
      dockerfile: Dockerfile
    container_name: rust-exporter
    environment:
      # 数据库
      - DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
      
      # 阿里云 Prometheus Pushgateway
      - PUSHGATEWAY_URL=https://workspace-default-cms-xxx.log.aliyuncs.com/prometheus/workspace-xxx/aliyun-prom-xxx/api/v1/pushgateway
      - PUSHGATEWAY_JOB=channel-health-exporter
      - PUSHGATEWAY_USERNAME=${ALIYUN_ACCESS_KEY_ID}
      - PUSHGATEWAY_PASSWORD=${ALIYUN_ACCESS_KEY_SECRET}
      - PUSHGATEWAY_INTERVAL=60
    ports:
      - "8001:8001"
      - "8002:8002"
    restart: unless-stopped
    networks:
      - monitoring

  # 本地 Prometheus（用于测试和 Grafana）
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - monitoring
    restart: unless-stopped

  # Grafana（连接本地 Prometheus）
  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    environment:
      - GF_SECURITY_ADMIN_USER=${GRAFANA_ADMIN_USER:-admin}
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD:-admin}
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/var/lib/grafana/dashboards
      - grafana_data:/var/lib/grafana
    ports:
      - "3000:3000"
    networks:
      - monitoring
    restart: unless-stopped
    depends_on:
      - prometheus

networks:
  monitoring:
    driver: bridge

volumes:
  prometheus_data:
  grafana_data:
```

---

## 故障排查

### 问题 1: 推送失败 - 401 Unauthorized

**原因**：AccessKey 认证失败

**解决方案**：
1. 检查 AK/SK 是否正确
2. 确认 RAM 用户有 `AliyunPrometheusMetricWriteAccess` 权限
3. 测试 AK/SK：

```bash
curl -u 'your_ak:your_sk' \
  -H "Content-Type: text/plain; version=0.0.4; charset=utf-8" \
  --data-binary "test_metric 1.0" \
  "https://your-pushgateway-url/metrics/job/test"
```

### 问题 2: 推送失败 - 404 Not Found

**原因**：URL 格式错误

**检查**：
- URL 必须包含 `/api/v1/pushgateway`
- 确保没有多余的斜杠
- 使用正确的内网或公网地址

### 问题 3: 网络超时

**原因**：网络不通

**解决方案**：
1. 如果在阿里云内部署，使用内网地址
2. 检查安全组规则
3. 检查 VPC 网络配置
4. 测试网络连通性：

```bash
curl -I https://your-pushgateway-url
```

### 问题 4: 数据未显示

**原因**：数据延迟或查询错误

**排查步骤**：
1. 等待 1-2 分钟（数据有延迟）
2. 检查 Job 名称是否正确
3. 在阿里云控制台查看 Pushgateway 日志
4. 确认指标名称拼写正确

---

## 性能和成本

### 推送频率建议

```bash
# 标准频率（推荐）
PUSHGATEWAY_INTERVAL=60  # 每分钟推送一次

# 高频率（实时性要求高）
PUSHGATEWAY_INTERVAL=30  # 每 30 秒推送一次

# 低频率（降低成本）
PUSHGATEWAY_INTERVAL=120  # 每 2 分钟推送一次
```

### 网络流量

- 每次推送约 1-2 KB
- 60 秒间隔：~1.4 MB/天
- 30 秒间隔：~2.8 MB/天

### 成本优化

1. **使用内网地址**：免流量费
2. **合理设置间隔**：60-120 秒足够
3. **只推送必要指标**：减少数据量

---

## 完整配置示例

### .env 文件

```bash
# 数据库
DATABASE_URL=postgres://dev_read_chunqiu:w7QcN8zp2VxT5Rb@pgm-6we2iwj50gul9ez8eo.pgsql.japan.rds.aliyuncs.com:5432/claude_code

# 阿里云 Prometheus Pushgateway（内网地址）
PUSHGATEWAY_URL=https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1-intranet.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway
PUSHGATEWAY_JOB=channel-health-exporter
PUSHGATEWAY_USERNAME=LTAI5tXXXXXXXXXX
PUSHGATEWAY_PASSWORD=your_secret_key
PUSHGATEWAY_INTERVAL=60

# Grafana
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=admin
```

### 启动命令

```bash
# 1. 配置环境变量
vim .env

# 2. 构建镜像
docker compose build rust-exporter

# 3. 启动服务
docker compose up -d

# 4. 查看日志
docker compose logs -f rust-exporter

# 5. 验证推送
# 等待 1-2 分钟后，在阿里云控制台查询指标
```

---

## 双模式运行

### 架构

```
┌─────────────────────────────────────────────────┐
│                Rust Exporter                     │
│                                                  │
│  ┌──────────────┐         ┌─────────────────┐  │
│  │  Collectors  │────────▶│  Metrics Store  │  │
│  │  (60s 间隔)   │         │  (in-memory)    │  │
│  └──────────────┘         └─────────────────┘  │
│         │                         │             │
│         │                         │             │
│         ▼                         ▼             │
│  ┌──────────────┐         ┌─────────────────┐  │
│  │ Pushgateway  │         │  /metrics API   │  │
│  │   Worker     │         │  (HTTP 8001)    │  │
│  │  (60s 推送)   │         └─────────────────┘  │
│  └──────────────┘                 │             │
└─────────────────────────────────────────────────┘
         │                           │
         │                           │
         ▼                           ▼
┌─────────────────┐         ┌─────────────────┐
│  阿里云 Prometheus│         │ 本地 Prometheus  │
│   (生产环境)      │         │   (测试/备份)     │
└─────────────────┘         └─────────────────┘
         │                           │
         │                           │
         └───────────┬───────────────┘
                     │
                     ▼
            ┌─────────────────┐
            │     Grafana     │
            │  (可切换数据源)   │
            └─────────────────┘
```

### 优势

- ✅ 生产数据推送到阿里云（高可用、免维护）
- ✅ 本地 Prometheus 作为备份（应急使用）
- ✅ 本地测试方便（curl localhost:8001/metrics）
- ✅ Grafana 可以灵活切换数据源

---

## 迁移步骤

### 从本地 Prometheus 迁移到阿里云

1. **添加 Pushgateway 配置**（不删除本地 Prometheus）
   ```bash
   vim .env
   # 添加 PUSHGATEWAY_* 配置
   ```

2. **重启服务**
   ```bash
   docker compose restart rust-exporter
   ```

3. **观察 1-2 天**
   - 检查日志确认推送成功
   - 在阿里云控制台验证数据
   - 对比本地和阿里云数据一致性

4. **切换 Grafana 数据源**
   - Configuration > Data Sources
   - 添加阿里云 Prometheus 数据源
   - 测试连接
   - 修改 Dashboard 使用新数据源

5. **（可选）停止本地 Prometheus**
   ```bash
   docker compose stop prometheus
   ```

### 回滚方案

如果阿里云出现问题：

```bash
# 1. 停止推送
unset PUSHGATEWAY_URL
docker compose restart rust-exporter

# 2. Grafana 切回本地
# Configuration > Data Sources > 选择本地 Prometheus
```

---

## 总结

**推荐配置**（生产环境）：

```bash
# 主要推送到阿里云
PUSHGATEWAY_URL=https://workspace-xxx-intranet.log.aliyuncs.com/prometheus/xxx/api/v1/pushgateway
PUSHGATEWAY_USERNAME=your_ak
PUSHGATEWAY_PASSWORD=your_sk
PUSHGATEWAY_INTERVAL=60

# 保留本地 Prometheus 作为备份
# docker-compose.yml 中保留 prometheus 服务
```

**优势**：
- ✅ 生产数据在阿里云（高可用、免维护）
- ✅ 本地有备份（应急）
- ✅ 开发测试方便
- ✅ 使用内网地址免流量费

---

**相关文档**：
- [DEPLOYMENT.md](./DEPLOYMENT.md) - 完整部署指南
- [E2E_VALIDATION.md](./E2E_VALIDATION.md) - 端到端验证
