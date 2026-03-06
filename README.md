# 渠道健康监控系统

基于 Prometheus + Grafana 的渠道健康监控系统，实时监控渠道可用性、缓存复用率和成本。

## 目录

- [架构](#架构)
- [测试](#测试)
- [部署](#部署)
- [开发流程](#开发流程)

---

## 架构

### 整体架构

本地只运行 Rust Exporter，Prometheus 和 Grafana 使用线上实例：

```
┌──────────────────┐
│    PostgreSQL    │
│  (阿里云 RDS)     │
└────────┬─────────┘
         │ SQL 查询
         ▼
┌──────────────────┐     Pushgateway      ┌──────────────────┐
│  Rust Exporter   │ ──────推送指标──────▶ │   Prometheus     │
│  (本地 Docker)    │                      │  (阿里云托管)     │
│  :8001/metrics   │                      └────────┬─────────┘
└──────────────────┘                               │ PromQL
                                                   ▼
┌──────────────────┐     Grafana API      ┌──────────────────┐
│  看板 JSON       │ ──────推送看板──────▶ │    Grafana       │
│  (本地文件)       │                      │  (线上实例)       │
└──────────────────┘                      └──────────────────┘
```

### 数据流

**指标数据**：PostgreSQL → Rust Exporter → Pushgateway → 阿里云 Prometheus → 线上 Grafana

**看板配置**：本地 JSON → Grafana API → 线上 Grafana

### 组件详情

#### Rust Exporter（本地唯一运行的服务）

高性能指标采集器，基于 Rust + Axum + SQLx：

```
rust-exporter/src/
├── main.rs               # 入口
├── db.rs                 # 数据库连接池
├── pushgateway.rs        # 推送指标到阿里云 Prometheus
├── api/                  # HTTP 服务（/metrics, /health）
├── core/
│   └── collector.rs      # MetricCollector trait
└── metrics/
    ├── availability/     # 渠道可用性采集
    ├── cache/            # 缓存复用率采集
    └── cost/             # 成本采集
```

#### 线上服务（不在本地运行）

- **Grafana**：`https://grafana.aicodewith.com`，通过 API 管理看板和数据源
- **Prometheus**：阿里云托管，Exporter 通过 Pushgateway 推送指标
- **PostgreSQL**：阿里云 RDS，Exporter 和 Grafana SQL 看板共用

### 监控指标体系

| 指标 | Prometheus Metric | 含义 |
|------|-------------------|------|
| 渠道可用性 | `channel_availability_percent{channel_group}` | 排除用户错误后的成功率 |
| 缓存复用率 | `channel_cache_reuse_percent{channel_group}` | 缓存命中率，越高越省钱 |
| Opus 成本 | `channel_avg_cost_cny_opus{channel_group}` | Opus 模型平均请求成本 |
| Sonnet 成本 | `channel_avg_cost_cny_sonnet{channel_group}` | Sonnet 模型平均请求成本 |
| 综合成本 | `channel_avg_cost_cny_all{channel_group}` | 所有模型平均请求成本 |

所有指标按 `channel_group` 标签分组：`aws`（AWS 渠道）、`special`（特价渠道）。

---

## 测试

### 单元测试

验证 Rust Exporter 的采集逻辑：

```bash
cd rust-exporter
cargo test
```

### E2E 截图测试

使用 Playwright 对线上 Grafana 截图验证看板渲染：

```bash
# 安装（首次）
npm ci
npx playwright install chromium --with-deps

# 运行
npx playwright test

# 查看报告
npx playwright show-report
```

截图保存在 `test-results/screenshots/`。

---

## 部署

### 本地开发

```bash
# 1. 配置环境变量
cp .env.example .env
# 编辑 .env，填入数据库和 Pushgateway 连接信息

# 2. 启动 Exporter
docker compose up -d

# 3. 验证
curl http://localhost:8001/health
curl http://localhost:8001/metrics | grep channel_
```

### 环境变量

```bash
# 数据库（必填）
DB_HOST=your-database-host
DB_PORT=5432
DB_NAME=your-database-name
DB_USER=your-database-user
DB_PASSWORD=your-database-password

# 阿里云 Prometheus Pushgateway（生产必填）
PUSHGATEWAY_URL=https://xxx.log.aliyuncs.com/.../api/v1/pushgateway
PUSHGATEWAY_JOB=channel-health-exporter
PUSHGATEWAY_USERNAME=your-access-key-id
PUSHGATEWAY_PASSWORD=your-access-key-secret
PUSHGATEWAY_INTERVAL=60
```

### 停止

```bash
docker compose down
```

---

## 开发流程

### 新增监控指标的标准流程

```
第一步：SQL 看板验证             第二步：转 Exporter            第三步：部署上线
┌─────────────────────┐      ┌─────────────────────┐      ┌─────────────────────┐
│  Grafana SQL 看板    │      │  Rust Exporter       │      │  Pushgateway 推送    │
│  直接查 PostgreSQL   │ ──▶  │  去掉 GROUP BY time  │ ──▶  │  阿里云 Prometheus   │
│                      │      │  只保留"当前值"查询   │      │  一天后图就完整       │
│  ✅ 历史数据立刻可见   │      │  ✅ 逻辑验证过了      │      │  ✅ 生产可用          │
└─────────────────────┘      └─────────────────────┘      └─────────────────────┘
```

### 第一步：用 SQL 看板调试指标

1. 构建看板 JSON，SQL 直接查 PostgreSQL 历史数据
2. 通过 Grafana API 推送到线上
3. Playwright 截图验证 → 有问题自动修改 → 没问题交给用户验收

```sql
-- 示例：过去 7 天每小时的渠道可用性
SELECT
  date_trunc('hour', created_at) AS time,
  channel_group,
  COUNT(*) FILTER (WHERE status < 400) * 100.0 / COUNT(*) AS availability
FROM requests
WHERE created_at > now() - interval '7 days'
  AND $__timeFilter(created_at)
GROUP BY 1, 2
ORDER BY 1
```

### 第二步：转成 Exporter

确认 SQL 逻辑没问题后，去掉时间维度，只查"当前窗口"的值：

```sql
-- 转换前（SQL 看板，带时间维度）
SELECT date_trunc('hour', created_at) AS time, avg(cost) ...
GROUP BY 1

-- 转换后（Exporter，只要当前值）
SELECT avg(cost) ...
WHERE created_at > now() - interval '5 minutes'
```

**转换规则**：去掉 `GROUP BY time` 和 `$__timeFilter()`，加上 `WHERE created_at > now() - interval 'N minutes'`。时间序列由 Prometheus 按采集周期自动生成。

### 第三步：部署，等数据积累

部署后 Exporter 通过 Pushgateway 推送指标到阿里云 Prometheus，一天后 Grafana 就有完整曲线。

### 为什么分两步

| | SQL 看板 | Exporter + Prometheus |
|---|---|---|
| 能看历史数据 | ✅ 立刻 | ❌ 需要积累 |
| 能做实时告警 | ❌ | ✅ |
| 适合开发调试 | ✅ | ❌ |
| 适合生产监控 | ❌ 每次查询压力大 | ✅ 轻量采集 |

---

## 项目结构

```
proxy_project-observatory/
├── AGENTS.md                          # AI 工作手册（含线上凭据）
├── docker-compose.yml                 # 只有 rust-exporter
├── .env.example                       # 环境变量模板
├── rust-exporter/                     # Rust 指标采集器
│   ├── Dockerfile
│   ├── src/
│   │   ├── main.rs
│   │   ├── db.rs
│   │   ├── pushgateway.rs             # 推送到阿里云 Prometheus
│   │   ├── api/                       # HTTP 端点
│   │   ├── core/collector.rs          # MetricCollector trait
│   │   └── metrics/                   # 三个采集器
│   └── tests/
├── grafana/dashboards/                # 看板 JSON（通过 API 推线上）
├── e2e/                               # Playwright E2E 测试
├── playwright.config.ts
├── .github/workflows/ci.yml           # CI
└── docs/archive/                      # 归档（旧配置、迁移记录）
```

## 许可证

MIT
