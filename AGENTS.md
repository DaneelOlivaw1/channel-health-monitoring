# AGENTS.md — AI 开发工作手册

本文件供 AI Agent 使用，包含线上环境连接信息和完整开发工作流。

> **所有凭据存储在 `.env` 文件中，使用前先读取 `.env`。**

---

## 线上环境

所有连接信息从 `.env` 读取，变量定义如下：

### Grafana

```
URL:      $GRAFANA_URL
用户名:   $GRAFANA_USER
密码:      $GRAFANA_PASSWORD
```

### Prometheus

```
推送端点 (Pushgateway):  $PUSHGATEWAY_URL
读取端点 (Query):        $PROMETHEUS_QUERY_URL
认证用户名 (AccessKey):  $PUSHGATEWAY_USERNAME
认证密码:                $PUSHGATEWAY_PASSWORD
```

### 数据库（只读）

```
Host:     $DB_HOST
Port:     $DB_PORT
DB:       $DB_NAME
User:     $DB_USER
Password: $DB_PASSWORD
```

---

## 开发工作流

### 总览

```
用户提需求
    │
    ▼
┌──────────────────────────────────────────────────────────────────┐
│  第一步：SQL 看板开发                                              │
│                                                                    │
│  1. 写 SQL 查询，验证指标逻辑                                       │
│  2. 构建 Grafana 看板 JSON                                         │
│  3. 通过 Grafana API 推送到线上                                     │
│  4. 用 Playwright 截图验证看板渲染                                   │
│  5. 有问题 → 自动修改 JSON → 重新推送 → 重新截图                      │
│  6. 没问题 → 截图交给用户验收                                        │
└──────────────────────────────────────────────────────────────────┘
    │ 用户验收通过
    ▼
┌──────────────────────────────────────────────────────────────────┐
│  第二步：Exporter 开发                                             │
│                                                                    │
│  1. 把 SQL 转成 Exporter（去掉 GROUP BY time，只查当前窗口）          │
│  2. cargo test 验证                                                │
│  3. 部署 Exporter，推送指标到线上 Prometheus                         │
│  4. 用 Playwright 截图验证 Prometheus 看板                           │
│  5. 交给用户最终验收                                                 │
└──────────────────────────────────────────────────────────────────┘
```

### 第一步：SQL 看板开发（详细）

#### 1.1 确保 Grafana 有 PostgreSQL 数据源

```bash
# 检查是否已有 PostgreSQL 数据源
curl -u $GRAFANA_USER:$GRAFANA_PASSWORD $GRAFANA_URL/api/datasources | jq '.[] | select(.type=="postgres")'

# 如果没有，创建一个
curl -u $GRAFANA_USER:$GRAFANA_PASSWORD -X POST $GRAFANA_URL/api/datasources \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "PostgreSQL-ReadOnly",
    "type": "postgres",
    "url": "'$DB_HOST':'$DB_PORT'",
    "database": "'$DB_NAME'",
    "user": "'$DB_USER'",
    "secureJsonData": { "password": "'$DB_PASSWORD'" },
    "access": "proxy",
    "jsonData": {
      "sslmode": "require",
      "maxOpenConns": 5,
      "maxIdleConns": 2,
      "connMaxLifetime": 14400,
      "postgresVersion": 1200,
      "timescaledb": false
    }
  }'
```

#### 1.2 构建看板 JSON

看板 JSON 放在 `grafana/dashboards/` 目录下。面板的 SQL 查询示例：

```sql
-- 时间序列查询（SQL 看板用，带 GROUP BY time）
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

#### 1.3 推送看板到线上 Grafana

```bash
# 推送/更新看板
curl -u $GRAFANA_USER:$GRAFANA_PASSWORD -X POST $GRAFANA_URL/api/dashboards/db \
  -H 'Content-Type: application/json' \
  -d '{
    "dashboard": <看板JSON内容>,
    "overwrite": true,
    "message": "AI 自动更新看板"
  }'

# 获取看板 URL
curl -u $GRAFANA_USER:$GRAFANA_PASSWORD $GRAFANA_URL/api/dashboards/uid/<dashboard-uid> | jq '.meta.url'
```

#### 1.4 Playwright 截图验证

```bash
# 对线上 Grafana 截图（环境变量从 .env 读取）
npx playwright test
```

截图保存在 `test-results/screenshots/`，检查：
- 面板是否有数据（没有 "No data"）
- 图表是否正确渲染
- 布局是否正常

#### 1.5 自动迭代

如果截图发现问题：
1. 分析截图，定位问题（SQL 错误 / 面板配置 / 布局）
2. 修改看板 JSON
3. 重新推送到 Grafana
4. 重新截图验证
5. 重复直到没问题

没问题后，把截图交给用户验收。

---

### 第二步：Exporter 开发（详细）

#### 2.1 SQL 转 Exporter

```sql
-- 转换前（SQL 看板）
SELECT date_trunc('hour', created_at) AS time, avg(cost) ...
GROUP BY 1

-- 转换后（Exporter）
SELECT avg(cost) ...
WHERE created_at > now() - interval '5 minutes'
```

转换规则：
- 去掉 `GROUP BY time`
- 去掉 `$__timeFilter()`
- 加上 `WHERE created_at > now() - interval 'N minutes'`
- 结果是单个数值 → 对应一个 Gauge

#### 2.2 修改 Rust Exporter

代码位置：`rust-exporter/src/metrics/`

每个采集器实现 `MetricCollector` trait：
```rust
pub trait MetricCollector: Send + Sync {
    fn name(&self) -> &str;
    async fn collect(&self, pool: &PgPool) -> Result<()>;
}
```

#### 2.3 测试

```bash
cd rust-exporter
cargo test
```

#### 2.4 部署并验证

```bash
# 本地构建测试
docker compose up -d --build rust-exporter

# 检查指标端点
curl http://localhost:8001/metrics | grep <新指标名>

# 指标会通过 Pushgateway 自动推送到线上 Prometheus
# 等待一个采集周期（60s）后，在线上 Grafana 验证
```

---

## Grafana API 速查

```bash
# 所有操作都需要认证（变量从 .env 读取）
AUTH="-u $GRAFANA_USER:$GRAFANA_PASSWORD"
URL="$GRAFANA_URL"

# 看板操作
curl $AUTH "$URL/api/search"                              # 列出所有看板
curl $AUTH "$URL/api/dashboards/uid/<uid>"                 # 导出看板
curl $AUTH -X POST "$URL/api/dashboards/db" -H 'Content-Type: application/json' -d @dashboard.json  # 导入看板
curl $AUTH -X DELETE "$URL/api/dashboards/uid/<uid>"       # 删除看板

# 数据源操作
curl $AUTH "$URL/api/datasources"                          # 列出数据源
curl $AUTH -X POST "$URL/api/datasources" -H 'Content-Type: application/json' -d @datasource.json   # 添加数据源
curl $AUTH -X DELETE "$URL/api/datasources/<id>"           # 删除数据源

# 文件夹操作
curl $AUTH "$URL/api/folders"                               # 列出文件夹
curl $AUTH -X POST "$URL/api/folders" -H 'Content-Type: application/json' -d '{"title":"监控"}'     # 创建文件夹
```

---

## 项目结构（精简后）

```
proxy_project-observatory/
├── AGENTS.md                          # 本文件（AI 工作手册）
├── rust-exporter/                     # Exporter（本地唯一运行的服务）
│   ├── src/metrics/                   # 采集器代码
│   └── tests/                         # 单元测试
├── grafana/dashboards/                # 看板 JSON（通过 API 推送到线上）
├── e2e/                               # Playwright 测试（对线上 Grafana 截图）
└── .github/workflows/ci.yml          # CI
```

线上组件（不在本地运行）：
- **Grafana**：线上实例，通过 API 操作
- **Prometheus**：阿里云托管，Exporter 通过 Pushgateway 推送指标
- **PostgreSQL**：阿里云 RDS，Exporter 和 Grafana SQL 看板共用
