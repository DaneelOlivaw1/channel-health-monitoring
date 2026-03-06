# Rust Exporter 重写计划 v2（模块化架构）

## 📋 项目概述

将现有的 Python Prometheus Exporter 重写为 Rust 版本，采用**模块化架构**：
- 按业务领域分类指标（Availability、Cost、Cache）
- 统一的 `MetricCollector` trait
- 可插拔的模块设计
- 自动生成 Prometheus 指标和 API 文档

---

## 🎯 目标

### 功能目标
- ✅ 保持现有 5 个指标的功能不变
- ✅ 提供 `/metrics` endpoint（Prometheus 格式）
- ✅ 提供 `/metrics-metadata` endpoint（JSON API）
- ✅ 提供 `/swagger-ui` endpoint（交互式文档）
- ✅ 支持 PostgreSQL 数据采集
- ✅ **模块化架构**：易于扩展新指标类别

### 技术目标
- ✅ 使用 `#[derive(Metrics)]` 自动生成指标
- ✅ 使用 `utoipa` 自动生成 OpenAPI 文档
- ✅ 编译时类型检查（无运行时错误）
- ✅ 性能提升 10x+（相比 Python）
- ✅ Docker 镜像体积减小 50%+
- ✅ **高内聚低耦合**：每个指标类别独立模块

---

## 🏗️ 技术栈

```toml
[dependencies]
# Web 框架
axum = "0.7"
tokio = { version = "1", features = ["full"] }

# Prometheus 指标
metrics = "0.23"
metrics-exporter-prometheus = "0.15"
metrics-derive = "0.1"

# API 文档
utoipa = { version = "5.4", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "8.0", features = ["axum"] }

# 数据库
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls"] }

# 异步
async-trait = "0.1"

# 配置和日志
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
```

---

## 📁 项目结构（模块化）

```
rust-exporter/
├── Cargo.toml
├── Dockerfile
├── .env.example
├── src/
│   ├── main.rs                    # 主程序：组装所有模块
│   │
│   ├── core/                      # 核心抽象
│   │   ├── mod.rs
│   │   ├── collector.rs           # MetricCollector trait
│   │   ├── registry.rs            # 指标注册中心
│   │   └── config.rs              # 配置管理
│   │
│   ├── metrics/                   # 指标模块（按类别）
│   │   ├── mod.rs
│   │   │
│   │   ├── availability/          # 可用性指标
│   │   │   ├── mod.rs
│   │   │   ├── metrics.rs         # #[derive(Metrics)]
│   │   │   └── collector.rs       # 实现 MetricCollector
│   │   │
│   │   ├── cost/                  # 成本指标
│   │   │   ├── mod.rs
│   │   │   ├── metrics.rs
│   │   │   └── collector.rs
│   │   │
│   │   └── cache/                 # 缓存指标
│   │       ├── mod.rs
│   │       ├── metrics.rs
│   │       └── collector.rs
│   │
│   ├── api/                       # API 端点
│   │   ├── mod.rs
│   │   ├── metrics.rs             # /metrics
│   │   ├── metadata.rs            # /metrics-metadata
│   │   └── docs.rs                # OpenAPI
│   │
│   └── db.rs                      # 数据库连接池
│
└── tests/
    ├── availability_tests.rs
    ├── cost_tests.rs
    └── cache_tests.rs
```

---

## 🔄 实现步骤

### Phase 1: 项目初始化（1-2 小时）
- [ ] 创建项目：`cargo new rust-exporter`
- [ ] 添加依赖到 `Cargo.toml`
- [ ] 创建模块化目录结构
- [ ] 配置 `.gitignore` 和 `rustfmt.toml`

### Phase 2: 核心抽象（2-3 小时）
- [ ] 实现 `core/collector.rs`：
  ```rust
  #[async_trait]
  pub trait MetricCollector: Send + Sync {
      fn name(&self) -> &'static str;
      async fn collect(&self, pool: &PgPool) -> Result<()>;
      fn interval(&self) -> u64 { 60 }
      fn enabled(&self) -> bool { true }
  }
  ```
- [ ] 实现 `core/config.rs`：环境变量配置
- [ ] 实现 `db.rs`：PostgreSQL 连接池

### Phase 3: Availability 模块（2-3 小时）
- [ ] 实现 `metrics/availability/metrics.rs`：
  ```rust
  #[derive(Metrics, Clone)]
  #[metrics(scope = "channel_availability")]
  pub struct AvailabilityMetrics {
      /// 渠道可用性百分比（排除用户错误）
      #[metric(labels = ["channel_group"])]
      pub percent: Gauge,
  }
  ```
- [ ] 实现 `metrics/availability/collector.rs`：
  - 实现 `MetricCollector` trait
  - SQL 查询可用性数据
- [ ] 单元测试

### Phase 4: Cost 和 Cache 模块（3-4 小时）
- [ ] 实现 `metrics/cost/` 模块
- [ ] 实现 `metrics/cache/` 模块
- [ ] 单元测试各模块

### Phase 5: API 端点（3-4 小时）
- [ ] 实现 `/metrics` endpoint
- [ ] 实现 `/metrics-metadata` endpoint（从各模块收集元数据）
- [ ] 实现 OpenAPI 文档生成

### Phase 6: 主程序（2-3 小时）
- [ ] 实现 `main.rs`：
  ```rust
  let collectors: Vec<Box<dyn MetricCollector>> = vec![
      Box::new(AvailabilityCollector::new()),
      Box::new(CostCollector::new()),
      Box::new(CacheCollector::new()),
  ];
  
  // 每个采集器独立 tokio task
  for collector in collectors {
      tokio::spawn(async move {
          // 定时采集
      });
  }
  ```
- [ ] 优雅关闭处理

### Phase 7: Docker 化（1-2 小时）
- [ ] 多阶段构建 Dockerfile
- [ ] 更新 `docker-compose.yml`
- [ ] 测试构建和运行

### Phase 8: 测试和验证（2-3 小时）
- [ ] 单元测试（每个模块独立）
- [ ] 集成测试
- [ ] 性能测试
- [ ] 功能验证

### Phase 9: 文档和部署（1-2 小时）
- [ ] 更新 `README.md`
- [ ] 编写迁移指南
- [ ] 性能对比报告

---

## 📊 架构优势

### 对比单文件架构

| 特性 | 单文件架构 | 模块化架构 |
|------|-----------|-----------|
| **可维护性** | ❌ 所有代码混在一起 | ✅ 按类别清晰分离 |
| **可扩展性** | ❌ 添加新指标需改多处 | ✅ 只需添加新模块 |
| **可测试性** | ❌ 难以单独测试 | ✅ 每个模块独立测试 |
| **团队协作** | ❌ 容易冲突 | ✅ 不同人负责不同模块 |
| **性能** | ⚠️ 所有指标同时采集 | ✅ 可配置不同采集间隔 |

### 添加新指标类别（3 步）

```bash
# 1. 创建新模块
mkdir -p src/metrics/health

# 2. 定义指标和采集器
# src/metrics/health/metrics.rs
# src/metrics/health/collector.rs

# 3. 在 main.rs 注册
collectors.push(Box::new(HealthCollector::new()));
```

**完成！** 不需要修改其他任何代码。

---

## 📅 时间估算

| 阶段 | 预计时间 | 累计时间 |
|------|---------|---------|
| Phase 1: 项目初始化 | 1-2h | 2h |
| Phase 2: 核心抽象 | 2-3h | 5h |
| Phase 3: Availability 模块 | 2-3h | 8h |
| Phase 4: Cost 和 Cache 模块 | 3-4h | 12h |
| Phase 5: API 端点 | 3-4h | 16h |
| Phase 6: 主程序 | 2-3h | 19h |
| Phase 7: Docker 化 | 1-2h | 21h |
| Phase 8: 测试和验证 | 2-3h | 24h |
| Phase 9: 文档和部署 | 1-2h | 26h |

**总计**：约 26 小时（3-4 个工作日）

---

## 🚀 下一步

1. **Review 这个计划**：确认模块化架构
2. **创建 Rust 项目**：`cargo new rust-exporter`
3. **开始 Phase 1**：项目初始化

详细架构设计见：`RUST_MODULAR_ARCHITECTURE.md`

准备好了吗？我可以开始实现！
