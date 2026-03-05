# 渠道健康监控部署指南

## 架构说明

已将原有的 PostgreSQL 直连看板改造为 Prometheus + Grafana 架构：

- **channel-health-exporter**: Python exporter，每 60 秒从 PostgreSQL 采集业务指标
- **Prometheus**: 抓取 exporter 指标并存储时序数据
- **Grafana**: 使用 PromQL 查询 Prometheus 数据并可视化

## 采集的指标

### 聚合指标（最近 3 小时）

- `channel_availability_percent{channel_group="aws|special"}` - 渠道可用性（%）
- `channel_cache_reuse_percent{channel_group="aws|special"}` - 缓存复用率（%）
- `channel_avg_cost_cny_opus{channel_group="aws|special"}` - Opus 平均成本（¥）
- `channel_avg_cost_cny_sonnet{channel_group="aws|special"}` - Sonnet 平均成本（¥）
- `channel_avg_cost_cny_all{channel_group="aws|special"}` - 所有模型平均成本（¥）

### 标签说明

- `channel_group="special"`: 特价渠道（claude_laohu_max, claude_steven, claude_steven_az, claude_laohu_official）
- `channel_group="aws"`: AWS 渠道

## 启动服务

```bash
docker-compose up -d
```

## 访问地址

- **Grafana**: http://localhost:3000
  - 默认账号: admin / admin
  - 看板: "渠道健康状况 (Prometheus)"
  
- **Prometheus**: http://localhost:9090
  - 查看指标: http://localhost:9090/graph
  - 查看抓取目标: http://localhost:9090/targets
  
- **Channel Health Exporter**: http://localhost:8001/metrics
  - 直接查看原始指标

## 验证步骤

1. 检查 exporter 是否正常运行：
```bash
docker-compose logs channel-health-exporter
curl http://localhost:8001/metrics
```

2. 检查 Prometheus 是否抓取到数据：
- 访问 http://localhost:9090/targets
- 确认 `channel-health` job 状态为 UP
- 在 Graph 页面查询: `channel_availability_percent`

3. 检查 Grafana 看板：
- 访问 http://localhost:3000
- 导航到 "渠道健康状况 (Prometheus)" 看板
- 确认数据正常显示

## 故障排查

### Exporter 无法连接数据库

```bash
docker-compose logs channel-health-exporter
```

检查环境变量配置是否正确（docker-compose.yml 中的 DB_* 变量）。

### Prometheus 无法抓取数据

```bash
docker-compose logs prometheus
curl http://localhost:8001/metrics
```

确认 exporter 端口 8001 可访问，检查 prometheus.yml 配置。

### Grafana 看板无数据

1. 检查 Prometheus 数据源配置
2. 在 Grafana Explore 中手动执行 PromQL 查询
3. 确认时间范围设置正确（默认最近 3 小时）

## 配置调整

### 修改采集间隔

编辑 `docker-compose.yml`：

```yaml
channel-health-exporter:
  environment:
    - SCRAPE_INTERVAL=30  # 改为 30 秒
```

### 修改数据库连接

编辑 `docker-compose.yml` 中的 `DB_*` 环境变量。

### 修改 Prometheus 抓取间隔

编辑 `prometheus/prometheus.yml`：

```yaml
- job_name: 'channel-health'
  scrape_interval: 30s  # 改为 30 秒
```

## 公开看板设置

Grafana 支持将看板设置为公开访问（无需登录）：

1. 进入看板设置 → Sharing
2. 启用 "Public dashboard"
3. 复制公开链接分享

这就是为什么需要使用 Prometheus 而非直连 PostgreSQL 的原因。
