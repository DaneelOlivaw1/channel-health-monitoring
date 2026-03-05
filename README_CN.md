# Proxy Project Observatory

完整的监控技术栈，包含 Prometheus + Grafana + Alertmanager，预配置仪表板和自定义导出器。

## 功能特性

- **Prometheus**: 指标收集和存储
- **Grafana**: 可视化仪表板（系统概览、应用指标）
- **Alertmanager**: 告警路由和管理
- **Node Exporter**: 系统级指标（CPU、内存、磁盘、网络）
- **cAdvisor**: 容器指标
- **自定义导出器**: 应用特定指标示例

## 快速开始

### 前置要求

- Docker
- Docker Compose

### 启动技术栈

```bash
docker-compose up -d
```

### 访问服务

- **Grafana**: http://localhost:3000
  - 默认凭据: `admin` / `admin`
  - 预加载仪表板: 系统概览、应用指标

- **Prometheus**: http://localhost:9090
  - 探索指标和查询数据
  - 检查告警规则状态

- **Alertmanager**: http://localhost:9093
  - 查看活动告警
  - 配置告警路由

- **Node Exporter**: http://localhost:9100/metrics
- **cAdvisor**: http://localhost:8080
- **自定义导出器**: http://localhost:8000/metrics

### 停止技术栈

```bash
docker-compose down
```

删除数据卷（数据将丢失）:

```bash
docker-compose down -v
```

## 配置

### 环境变量

复制 `.env.example` 到 `.env` 并自定义:

```bash
cp .env.example .env
```

可用变量:
- `GRAFANA_ADMIN_USER`: Grafana 管理员用户名（默认: admin）
- `GRAFANA_ADMIN_PASSWORD`: Grafana 管理员密码（默认: admin）

### Prometheus 配置

编辑 `prometheus/prometheus.yml` 以:
- 添加抓取目标
- 调整抓取间隔
- 配置服务发现

### 告警规则

告警规则位于 `prometheus/alerts/rules.yml`:

- **主机告警**: CPU、内存、磁盘使用率
- **容器告警**: 容器健康状态和资源使用
- **应用告警**: 错误率、响应时间

根据需要编辑阈值和添加自定义规则。

### Alertmanager 配置

在 `alertmanager/alertmanager.yml` 中配置告警路由:

1. 更新 SMTP 设置以发送邮件通知
2. 配置 webhook 端点
3. 调整路由规则

### 自定义导出器

`exporters/custom-exporter/` 中的示例导出器演示了:

- 仪表指标（连接数、CPU、内存）
- 计数器指标（请求计数）
- 直方图指标（请求持续时间）

自定义方法:

1. 编辑 `exporters/custom-exporter/exporter.py`
2. 添加你的应用特定指标
3. 重新构建: `docker-compose up -d --build custom-exporter`

## 仪表板

### 系统概览

监控系统资源:
- CPU 使用率（当前 + 历史）
- 内存使用率（当前 + 历史）
- 磁盘空间可用性
- 网络流量
- 磁盘 I/O
- 服务状态

### 应用指标

监控应用性能:
- 错误率
- 响应时间百分位数（p50、p95、p99）
- 请求速率
- HTTP 状态码分布
- 活动连接数

## 告警规则

### 主机告警

| 告警 | 阈值 | 持续时间 | 严重程度 |
|-------|-----------|----------|----------|
| HostDown | 服务不可用 | 1分钟 | 严重 |
| HighCPUUsage | >80% | 5分钟 | 警告 |
| CriticalCPUUsage | >95% | 2分钟 | 严重 |
| HighMemoryUsage | >80% | 5分钟 | 警告 |
| CriticalMemoryUsage | >95% | 2分钟 | 严重 |
| HighDiskUsage | <20% 可用 | 5分钟 | 警告 |
| CriticalDiskUsage | <10% 可用 | 2分钟 | 严重 |

### 容器告警

| 告警 | 条件 | 持续时间 | 严重程度 |
|-------|-----------|----------|----------|
| ContainerDown | 容器未运行 | 1分钟 | 警告 |
| HighContainerCPU | >80% | 5分钟 | 警告 |
| HighContainerMemory | >80% | 5分钟 | 警告 |

### 应用告警

| 告警 | 阈值 | 持续时间 | 严重程度 |
|-------|-----------|----------|----------|
| HighErrorRate | >5% | 5分钟 | 警告 |
| SlowResponseTime | p95 >1秒 | 5分钟 | 警告 |

## 扩展技术栈

### 添加新的抓取目标

编辑 `prometheus/prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'my-service'
    static_configs:
      - targets: ['my-service:9090']
        labels:
          service: 'my-service'
```

### 添加自定义告警规则

在 `prometheus/alerts/` 中创建新文件:

```yaml
groups:
  - name: my_alerts
    interval: 30s
    rules:
      - alert: MyAlert
        expr: my_metric > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "我的自定义告警"
```

### 导入额外的仪表板

1. 从 Grafana UI 导出仪表板 JSON
2. 保存到 `grafana/dashboards/`
3. 重启 Grafana: `docker-compose restart grafana`

## 故障排查

### 检查服务日志

```bash
docker-compose logs prometheus
docker-compose logs grafana
docker-compose logs alertmanager
```

### 验证目标

打开 Prometheus UI (http://localhost:9090) → Status → Targets

所有目标应显示 "UP" 状态。

### 重新加载 Prometheus 配置

```bash
curl -X POST http://localhost:9090/-/reload
```

### 仪表板无法加载

1. 检查 Grafana 日志: `docker-compose logs grafana`
2. 验证仪表板 JSON 语法
3. 重启 Grafana: `docker-compose restart grafana`

## 项目结构

```
proxy_project-observatory/
├── docker-compose.yml
├── .env.example
├── prometheus/
│   ├── prometheus.yml
│   └── alerts/
│       └── rules.yml
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   └── prometheus.yml
│   │   └── dashboards/
│   │       └── default.yml
│   └── dashboards/
│       ├── system-overview.json
│       └── application-metrics.json
├── alertmanager/
│   └── alertmanager.yml
└── exporters/
    └── custom-exporter/
        ├── Dockerfile
        ├── requirements.txt
        └── exporter.py
```

## 许可证

MIT
