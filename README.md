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

系统由四个组件组成，通过 Docker Compose 编排：

```
                        ┌──────────────┐
                        │  PostgreSQL  │
                        │  (外部数据库)  │
                        └──────┬───────┘
                               │ SQL 查询
                               ▼
┌─────────────────────── Docker Compose ───────────────────────┐
│                                                               │
│  ┌─────────────────────┐                                      │
│  │   Rust Exporter     │                                      │
│  │   :8001/metrics     │──── Prometheus 格式 ──┐              │
│  │   :8001/health      │                        │              │
│  └─────────────────────┘                        ▼              │
│                                        ┌──────────────────┐   │
│                                        │   Prometheus     │   │
│                                        │   :9090          │   │
│                                        │   每 60s 抓取     │   │
│                                        └────────┬─────────┘   │
│                                                  │ PromQL      │
│                                                  ▼              │
│                                        ┌──────────────────┐   │
│                                        │    Grafana       │   │
│                                        │    :3000         │   │
│                                        │    可视化看板     │   │
│                                        └──────────────────┘   │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 数据流

```
PostgreSQL ──SQL──▶ Rust Exporter ──/metrics──▶ Prometheus ──PromQL──▶ Grafana
                   (采集+计算)        (抓取)      (存储+查询)    (查询)    (展示)
```

1. **Rust Exporter** 连接外部 PostgreSQL，执行 SQL 查询计算业务指标
2. **Prometheus** 每 60 秒抓取 Exporter 的 `/metrics` 端点
3. **Grafana** 通过 PromQL 从 Prometheus 查询数据，渲染看板

### 组件详情

#### Rust Exporter（Rust + Axum + SQLx）

高性能指标采集器，替代原有 Python 版本：

| 指标 | Python | Rust | 提升 |
|------|--------|------|------|
| 启动时间 | ~2s | ~0.2s | 10x |
| 内存占用 | ~50MB | ~10MB | 5x |
| CPU（空闲） | ~2% | ~0.5% | 4x |

代码结构：

```
rust-exporter/src/
├── main.rs               # 入口
├── db.rs                 # 数据库连接池
├── api/                  # HTTP 服务（Axum）
├── core/
│   └── collector.rs      # MetricCollector trait
└── metrics/
    ├── availability/     # 渠道可用性采集
    ├── cache/            # 缓存复用率采集
    └── cost/             # 成本采集
```

#### Prometheus

```yaml
# prometheus/prometheus.yml
scrape_configs:
  - job_name: 'rust-exporter'
    scrape_interval: 60s
    static_configs:
      - targets: ['rust-exporter:8001']
```

#### Grafana

通过 provisioning 自动配置数据源和看板，无需手动操作：

```
grafana/
├── provisioning/
│   ├── datasources/prometheus.yml    # 自动注册 Prometheus 数据源
│   └── dashboards/default.yml        # 自动加载看板
└── dashboards/
    ├── channel-health-prometheus.json # 渠道健康看板
    └── metrics-registry.json          # 指标注册看板
```

### 监控指标体系

| 指标 | Prometheus Metric | 含义 |
|------|-------------------|------|
| 渠道可用性 | `channel_availability_percent{channel_group}` | 排除用户错误（400/404/413/429）和鉴权问题（401/403）后的成功率 |
| 缓存复用率 | `channel_cache_reuse_percent{channel_group}` | 缓存读取量 / (缓存读取量 + 缓存创建量)，越高越省钱 |
| Opus 成本 | `channel_avg_cost_cny_opus{channel_group}` | Opus 模型平均请求成本（人民币） |
| Sonnet 成本 | `channel_avg_cost_cny_sonnet{channel_group}` | Sonnet 模型平均请求成本（人民币） |
| 综合成本 | `channel_avg_cost_cny_all{channel_group}` | 所有模型平均请求成本（人民币） |

所有指标按 `channel_group` 标签分组：`aws`（AWS 渠道）、`special`（特价渠道）。

---

## 测试

系统有三层测试，覆盖从代码到 UI 的完整链路：

```
┌─────────────────────────────────────────────────┐
│              E2E 截图测试（Playwright）            │  ← 验证看板渲染
├─────────────────────────────────────────────────┤
│              Smoke Test（CI 健康检查）             │  ← 验证服务启动
├─────────────────────────────────────────────────┤
│              单元测试（cargo test）               │  ← 验证采集逻辑
└─────────────────────────────────────────────────┘
```

### 第一层：单元测试

验证 Rust Exporter 的采集逻辑：

```bash
cd rust-exporter
cargo test
```

测试覆盖三个采集器的 SQL 查询和指标计算。

### 第二层：Smoke Test

CI 中启动全部服务，验证它们能正常运行和连通：

```bash
# 验证各服务健康
curl -f http://localhost:8001/health        # Rust Exporter
curl -f http://localhost:9090/-/healthy     # Prometheus
curl -f http://localhost:3000/api/health    # Grafana

# 验证指标端点
curl http://localhost:8001/metrics | grep channel_

# 验证 Prometheus 抓取目标
curl http://localhost:9090/api/v1/targets
```

### 第三层：E2E 截图测试

使用 Playwright 自动登录 Grafana，展开所有面板，截取看板截图。

#### 本地运行

```bash
# 先启动服务
docker compose up -d

# 安装 Playwright（首次）
npm ci
npx playwright install chromium --with-deps

# 运行测试
npx playwright test

# 查看报告
npx playwright show-report
```

#### 测试做了什么

```
1. 打开 Grafana → 登录（admin/admin）→ 跳过改密码
2. 导航到看板 → 展开所有折叠的 Row
3. 等待面板加载完成（networkidle + 面板可见性检查）
4. 截图保存到 test-results/screenshots/
```

具体截图：

| 测试 | 截图文件 | 验证内容 |
|------|----------|----------|
| 全页截图 | `dashboard-full.png` | 整个看板所有面板 |
| 可用性面板 | `panel-availability.png` | panel-21 渲染有数据 |
| 成本面板 | `panel-cost.png` | panel-23 渲染有数据 |
| 对比表格 | `panel-table.png` | panel-31 渲染有数据 |

#### 关键实现细节

**等待面板加载**——直接截图会截到"加载中"，需要等数据渲染完：

```typescript
// 等面板容器可见（Grafana 用 data-viz-panel-key 标记面板）
await page.locator('[data-viz-panel-key="panel-21"]').waitFor({ state: 'visible' });

// 等网络请求全部完成
await page.waitForLoadState('networkidle');

// 缓冲时间给动画和渲染
await page.waitForTimeout(2000);
```

**处理 Grafana 登录**——首次登录会弹"修改密码"弹窗：

```typescript
await page.locator('input[name="user"]').fill('admin');
await page.locator('input[name="password"]').fill('admin');
await page.locator('button[type="submit"]').click();

// 跳过改密码提示
const skipButton = page.locator('button', { hasText: /skip/i });
if (await skipButton.isVisible({ timeout: 3000 }).catch(() => false)) {
  await skipButton.click();
}
```

#### Playwright 配置

```typescript
// playwright.config.ts
export default defineConfig({
  testDir: './e2e',
  workers: 1,           // 单线程，避免并发问题
  retries: process.env.CI ? 2 : 0,  // CI 失败重试 2 次
  use: {
    baseURL: process.env.GRAFANA_URL || 'http://localhost:3000',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    trace: 'on-first-retry',
  },
});
```

---

## 部署

### 本地开发

```bash
# 1. 配置环境变量
cp .env.example .env
# 编辑 .env，填入数据库连接信息

# 2. 启动所有服务
docker compose up -d

# 3. 访问
# Grafana:   http://localhost:3000  (admin/admin)
# Prometheus: http://localhost:9090
# Exporter:   http://localhost:8001/metrics
```

### CI/CD（GitHub Actions）

CI 在每次 push/PR 时自动运行三个 Job：

```
┌──────────────┐   ┌──────────────┐   ┌────────────────────┐
│ rust-check   │   │ docker-build │──▶│   smoke-test       │
│ cargo check  │   │ compose build│   │   健康检查 + 指标验证 │
│ cargo test   │   └──────────────┘──▶│                    │
│ cargo clippy │                      ├────────────────────┤
└──────────────┘                      │ e2e-screenshots    │
                                      │ Playwright 截图测试 │
                                      └────────────────────┘
```

CI 配置位于 `.github/workflows/ci.yml`，关键步骤：

```yaml
# Smoke Test：验证服务启动和连通
- name: Verify metrics endpoint
  run: curl -sf http://localhost:8001/metrics

- name: Verify Prometheus targets
  run: curl -s http://localhost:9090/api/v1/targets

# E2E：Playwright 截图
- name: Install Playwright
  run: npx playwright install chromium --with-deps

- name: Run E2E screenshot tests
  run: npx playwright test

# 截图上传为 CI 产物，保留 30 天
- uses: actions/upload-artifact@v4
  if: always()
  with:
    name: dashboard-screenshots
    path: test-results/screenshots/
    retention-days: 30
```

E2E 测试中会等待首次指标抓取完成（sleep 65s），确保 Grafana 面板有数据可渲染。

截图产物可在 GitHub Actions 页面的 Artifacts 区域下载查看。

### 生产部署

#### 前置要求

- Docker + Docker Compose
- 可访问的 PostgreSQL 数据库

#### 步骤

```bash
# 1. 克隆项目
git clone <repo-url>
cd proxy_project-observatory

# 2. 配置环境变量
cp .env.example .env
```

编辑 `.env`：

```bash
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=<改成强密码>

DB_HOST=your-database-host
DB_PORT=5432
DB_NAME=your-database-name
DB_USER=your-database-user
DB_PASSWORD=your-database-password
```

```bash
# 3. 启动
docker compose up -d

# 4. 验证
docker compose ps                    # 所有服务 running
curl http://localhost:8001/health    # Exporter 健康
curl http://localhost:9090/-/healthy # Prometheus 健康
curl http://localhost:3000/api/health # Grafana 健康
```

#### 停止和清理

```bash
docker compose down      # 停止（保留数据）
docker compose down -v   # 停止并删除数据卷
```

#### 故障排查

```bash
# 查看日志
docker compose logs                        # 所有服务
docker compose logs rust-exporter          # 指定服务

# 验证指标采集
curl http://localhost:8001/metrics | grep channel_

# 验证 Prometheus 抓取
curl http://localhost:9090/api/v1/query?query=channel_availability_percent

# 检查 Grafana 数据源
# 访问 http://localhost:3000/connections/datasources
```

---

## 项目结构

```
proxy_project-observatory/
├── .github/workflows/ci.yml           # CI：编译检查 + Smoke Test + E2E 截图
├── docker-compose.yml                 # 服务编排
├── .env.example                       # 环境变量模板
├── rust-exporter/                     # Rust 指标采集器
│   ├── Dockerfile
│   ├── src/
│   │   ├── main.rs
│   │   ├── db.rs
│   │   ├── api/                       # HTTP 端点
│   │   ├── core/collector.rs          # MetricCollector trait
│   │   └── metrics/                   # 三个采集器
│   │       ├── availability/
│   │       ├── cache/
│   │       └── cost/
│   └── tests/                         # 单元测试
├── prometheus/
│   └── prometheus.yml                 # 抓取配置
├── grafana/
│   ├── provisioning/                  # 自动配置（数据源 + 看板加载）
│   └── dashboards/                    # 看板 JSON
├── e2e/
│   └── grafana-dashboard.spec.ts      # Playwright E2E 测试
├── playwright.config.ts               # Playwright 配置
└── docs/                              # 补充文档
```

---

## 开发流程

### 新增监控指标的标准流程

```
第一步：SQL 验证                第二步：转 Exporter            第三步：部署上线
┌─────────────────────┐      ┌─────────────────────┐      ┌─────────────────────┐
│  Grafana             │      │  Rust Exporter       │      │  Prometheus          │
│  + PostgreSQL 数据源  │ ──▶  │  去掉 GROUP BY time  │ ──▶  │  自动按时间采集       │
│                      │      │  只保留"当前值"查询   │      │  一天后图就完整       │
│  ✅ 历史数据立刻可见   │      │  ✅ 逻辑验证过了      │      │  ✅ 生产可用          │
└─────────────────────┘      └─────────────────────┘      └─────────────────────┘
```

### 第一步：用 SQL 看板调试指标

在 Grafana 中添加 PostgreSQL 数据源，直接写 SQL 查历史数据：

```sql
-- 示例：过去 7 天每小时的渠道可用性
SELECT
  date_trunc('hour', created_at) AS time,
  channel_group,
  COUNT(*) FILTER (WHERE status < 400) * 100.0 / COUNT(*) AS availability
FROM requests
WHERE created_at > now() - interval '7 days'
GROUP BY 1, 2
ORDER BY 1
```

这样可以 **立刻看到完整的历史趋势图**，快速验证指标逻辑是否正确。

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

**转换规则**：去掉 `GROUP BY time`，加上 `WHERE created_at > now() - interval 'N minutes'`。时间序列由 Prometheus 按采集周期自动生成，不需要你自己算。

### 第三步：部署，等数据积累

部署后 Prometheus 每 60 秒采一个点，一天后就有完整的 24 小时曲线。

### 为什么分两步

| | SQL 看板 | Exporter + Prometheus |
|---|---|---|
| 能看历史数据 | ✅ 立刻 | ❌ 需要积累 |
| 能做实时告警 | ❌ | ✅ |
| 适合开发调试 | ✅ | ❌ |
| 适合生产监控 | ❌ 每次查询压力大 | ✅ 轻量采集 |

这不是 workaround，是 **Prometheus 监控体系的标准做法**：Exporter 只回答"现在是多少"，时间序列由 Prometheus 负责积累。

---

## 许可证

MIT
