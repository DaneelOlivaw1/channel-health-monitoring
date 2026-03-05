# 渠道健康监控系统

基于 Prometheus + Grafana 的渠道健康监控系统，实时监控渠道可用性、缓存复用率和成本。

## 功能特性

- **Prometheus**: 指标收集和存储
- **Grafana**: 可视化看板，自动配置数据源和看板
- **Channel Health Exporter**: 从 PostgreSQL 采集业务指标

## 快速开始

### 前置要求

- Docker
- Docker Compose

### 配置数据库连接

1. 复制环境变量模板：

```bash
cp .env.example .env
```

2. 编辑 `.env` 文件，配置数据库连接信息：

```bash
# Grafana 管理员账号
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=admin

# 数据库连接信息
DB_HOST=your-database-host
DB_PORT=5432
DB_NAME=your-database-name
DB_USER=your-database-user
DB_PASSWORD=your-database-password
```

### 启动服务

```bash
docker compose up -d
```

### 访问服务

- **Grafana**: http://localhost:3000
  - 默认账号: `admin` / `admin`（可在 `.env` 中修改）
  - 看板会自动加载：渠道健康状况 (Prometheus)

- **Prometheus**: http://localhost:9090
  - 查看指标和查询数据
  - 检查抓取目标状态

- **Channel Health Exporter**: http://localhost:8001/metrics
  - 查看原始指标数据

### 停止服务

```bash
docker compose down
```

删除数据卷（数据将丢失）：

```bash
docker compose down -v
```

## 监控指标

### 渠道可用性
- 排除用户错误（400/404/413/429）和鉴权问题（401/403）后的成功率
- 按渠道分组：特价渠道、AWS 渠道

### 缓存复用率
- 缓存读取量 ÷（缓存读取量 + 缓存创建量）
- 越高越省钱

### 成本分析
- Opus 模型平均成本
- Sonnet 模型平均成本
- 所有模型平均成本

## 架构说明

```
┌─────────────────────────────────────────────────────────┐
│                     Docker Compose                       │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────────┐      ┌──────────────────┐        │
│  │   PostgreSQL     │      │   Prometheus     │        │
│  │   (外部数据库)    │◄─────│   (指标存储)      │        │
│  └──────────────────┘      └────────┬─────────┘        │
│           ▲                          │                   │
│           │                          │                   │
│           │                          ▼                   │
│  ┌────────┴──────────┐      ┌──────────────────┐        │
│  │ Channel Health    │      │     Grafana      │        │
│  │    Exporter       │      │   (可视化看板)    │        │
│  │  (指标采集器)      │      └──────────────────┘        │
│  └───────────────────┘                                   │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### 服务说明

1. **channel-health-exporter** (8001端口)
   - 每 60 秒从 PostgreSQL 采集业务指标
   - 暴露 `/metrics` 端点供 Prometheus 抓取

2. **prometheus** (9090端口)
   - 每 60 秒抓取 exporter 的指标
   - 存储时序数据

3. **grafana** (3000端口)
   - 自动配置 Prometheus 数据源
   - 自动加载渠道健康看板
   - 提供可视化界面

## 添加新的业务指标

如果需要监控其他业务指标（如用户指标、订单指标等），只需在 `exporters/channel-health-exporter/exporter.py` 中添加新的采集方法：

```python
# 定义新指标
user_count = Gauge("user_count", "Total user count", ["user_type"])

# 添加采集方法
def collect_user_metrics(self):
    conn = self.get_connection()
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT user_type, COUNT(*) FROM users GROUP BY user_type")
        for row in cursor.fetchall():
            user_type, count = row
            user_count.labels(user_type=user_type).set(count)
        cursor.close()
    finally:
        self.return_connection(conn)

# 在 collect_all_metrics 中调用
def collect_all_metrics(self):
    self.collect_availability_metrics()
    self.collect_cache_metrics()
    self.collect_cost_metrics()
    self.collect_user_metrics()  # 新增
```

重新构建并启动：

```bash
docker compose up -d --build channel-health-exporter
```

## 故障排查

### 检查服务状态

```bash
docker compose ps
```

### 查看日志

```bash
# 查看所有服务日志
docker compose logs

# 查看特定服务日志
docker compose logs channel-health-exporter
docker compose logs grafana
docker compose logs prometheus
```

### 验证指标采集

```bash
# 检查 exporter 是否正常暴露指标
curl http://localhost:8001/metrics | grep channel_

# 检查 Prometheus 是否抓取到数据
curl http://localhost:9090/api/v1/query?query=channel_availability_percent
```

### Grafana 看板不显示

1. 检查数据源是否配置成功：
   - 访问 http://localhost:3000/connections/datasources
   - 应该能看到 `prometheus` 数据源

2. 检查看板是否加载：
   - 访问 http://localhost:3000/dashboards
   - 应该能看到 "渠道健康状况 (Prometheus)"

3. 查看 Grafana 日志：
   ```bash
   docker compose logs grafana | grep -i "dashboard\|provision"
   ```

## 项目结构

```
channel-health-monitoring/
├── docker-compose.yml              # 服务编排配置
├── .env.example                    # 环境变量模板
├── .env                           # 环境变量（不提交到 Git）
├── README.md                      # 项目文档
├── docs/
│   └── CHANNEL_HEALTH_SETUP.md   # 部署文档（归档）
├── exporters/
│   └── channel-health-exporter/   # 渠道健康指标采集器
│       ├── Dockerfile
│       ├── requirements.txt
│       └── exporter.py
├── grafana/
│   ├── dashboards/                # 看板配置文件
│   │   └── channel-health-prometheus.json
│   └── provisioning/              # 自动配置
│       ├── datasources/
│       │   └── prometheus.yml    # 数据源配置
│       └── dashboards/
│           └── default.yml       # 看板加载配置
└── prometheus/
    └── prometheus.yml             # Prometheus 配置
```

## 许可证

MIT
