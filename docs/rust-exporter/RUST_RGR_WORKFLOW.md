# Rust Exporter RGR 开发流程

## 🔴🟢🔵 RGR (Red-Green-Refactor) 原则

### 核心循环
1. **🔴 Red**：先写测试，测试失败（红色）
2. **🟢 Green**：写最少的代码让测试通过（绿色）
3. **🔵 Refactor**：重构代码，保持测试通过

### 为什么用 RGR？
- ✅ 测试驱动设计（Test-Driven Design）
- ✅ 确保代码可测试性
- ✅ 防止过度设计
- ✅ 持续重构，保持代码质量
- ✅ 快速反馈循环

---

## 📋 RGR 实现步骤

### Iteration 1: 核心抽象 (MetricCollector Trait)

#### 🔴 Red: 写失败的测试
```rust
// tests/core_collector_tests.rs

use rust_exporter::core::collector::MetricCollector;
use sqlx::PgPool;

#[tokio::test]
async fn test_collector_trait_basic() {
    struct DummyCollector;
    
    #[async_trait]
    impl MetricCollector for DummyCollector {
        fn name(&self) -> &'static str {
            "dummy"
        }
        
        async fn collect(&self, _pool: &PgPool) -> anyhow::Result<()> {
            Ok(())
        }
    }
    
    let collector = DummyCollector;
    assert_eq!(collector.name(), "dummy");
    assert_eq!(collector.interval(), 60); // 默认值
    assert!(collector.enabled()); // 默认启用
}
```

**运行测试**：`cargo test` → ❌ 失败（trait 不存在）

#### 🟢 Green: 最小实现
```rust
// src/core/collector.rs

use async_trait::async_trait;
use sqlx::PgPool;
use anyhow::Result;

#[async_trait]
pub trait MetricCollector: Send + Sync {
    fn name(&self) -> &'static str;
    async fn collect(&self, pool: &PgPool) -> Result<()>;
    
    fn interval(&self) -> u64 {
        60
    }
    
    fn enabled(&self) -> bool {
        true
    }
}
```

**运行测试**：`cargo test` → ✅ 通过

#### 🔵 Refactor: 添加文档和类型
```rust
/// 指标采集器 trait
/// 
/// 每个指标类别实现这个 trait，实现自动注册和采集
#[async_trait]
pub trait MetricCollector: Send + Sync {
    /// 采集器名称（用于日志和错误报告）
    fn name(&self) -> &'static str;
    
    /// 采集指标数据
    async fn collect(&self, pool: &PgPool) -> Result<()>;
    
    /// 采集间隔（秒）
    fn interval(&self) -> u64 {
        60
    }
    
    /// 是否启用
    fn enabled(&self) -> bool {
        true
    }
}
```

**运行测试**：`cargo test` → ✅ 仍然通过

---

### Iteration 2: 数据库连接池

#### 🔴 Red: 写失败的测试
```rust
// tests/db_tests.rs

use rust_exporter::db::create_pool;

#[tokio::test]
async fn test_create_pool_success() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    let pool = create_pool(&database_url).await;
    assert!(pool.is_ok());
    
    let pool = pool.unwrap();
    let result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_pool_invalid_url() {
    let result = create_pool("invalid://url").await;
    assert!(result.is_err());
}
```

**运行测试**：`cargo test` → ❌ 失败（函数不存在）

#### 🟢 Green: 最小实现
```rust
// src/db.rs

use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

**运行测试**：`cargo test` → ✅ 通过

#### 🔵 Refactor: 添加配置和超时
```rust
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

**运行测试**：`cargo test` → ✅ 仍然通过

---

### Iteration 3: Availability 指标定义

#### 🔴 Red: 写失败的测试
```rust
// tests/availability_metrics_tests.rs

use rust_exporter::metrics::availability::AvailabilityMetrics;

#[test]
fn test_availability_metrics_creation() {
    let metrics = AvailabilityMetrics::default();
    
    // 测试指标可以设置值
    metrics.percent("aws").set(95.0);
    metrics.percent("special").set(98.5);
    
    // 验证指标名称正确
    // (这里需要通过 Prometheus 注册表验证)
}
```

**运行测试**：`cargo test` → ❌ 失败（结构体不存在）

#### 🟢 Green: 最小实现
```rust
// src/metrics/availability/metrics.rs

use metrics_derive::Metrics;
use metrics::Gauge;

#[derive(Metrics, Clone, Default)]
#[metrics(scope = "channel_availability")]
pub struct AvailabilityMetrics {
    #[metric(labels = ["channel_group"])]
    pub percent: Gauge,
}
```

**运行测试**：`cargo test` → ✅ 通过

#### 🔵 Refactor: 添加文档和更多指标
```rust
/// 可用性指标
#[derive(Metrics, Clone, Default)]
#[metrics(scope = "channel_availability")]
pub struct AvailabilityMetrics {
    /// 渠道可用性百分比（排除用户错误）
    /// 
    /// 计算公式：成功请求数 / (总请求数 - 用户错误请求数) × 100
    #[metric(labels = ["channel_group"])]
    pub percent: Gauge,
    
    /// 总请求数
    #[metric(labels = ["channel_group"])]
    pub total_requests: Counter,
    
    /// 成功请求数
    #[metric(labels = ["channel_group"])]
    pub successful_requests: Counter,
}
```

**运行测试**：`cargo test` → ✅ 仍然通过

---

### Iteration 4: Availability 采集器

#### 🔴 Red: 写失败的测试
```rust
// tests/availability_collector_tests.rs

use rust_exporter::metrics::availability::AvailabilityCollector;
use rust_exporter::core::collector::MetricCollector;
use sqlx::PgPool;

async fn setup_test_db() -> PgPool {
    let pool = create_test_pool().await;
    
    // 插入测试数据
    sqlx::query!(
        r#"
        INSERT INTO channel_request_log (channel_code, status, response_code, created_at)
        VALUES 
            ('aws', 'success', 200, NOW() - INTERVAL '1 hour'),
            ('aws', 'failed', 500, NOW() - INTERVAL '1 hour'),
            ('special', 'success', 200, NOW() - INTERVAL '1 hour')
        "#
    )
    .execute(&pool)
    .await
    .unwrap();
    
    pool
}

#[tokio::test]
async fn test_availability_collector_collect() {
    let pool = setup_test_db().await;
    let collector = AvailabilityCollector::new();
    
    let result = collector.collect(&pool).await;
    assert!(result.is_ok());
    
    // 验证指标值被更新
    // (需要通过 Prometheus 注册表读取)
}

#[tokio::test]
async fn test_availability_collector_name() {
    let collector = AvailabilityCollector::new();
    assert_eq!(collector.name(), "availability");
}
```

**运行测试**：`cargo test` → ❌ 失败（采集器不存在）

#### 🟢 Green: 最小实现
```rust
// src/metrics/availability/collector.rs

use super::metrics::AvailabilityMetrics;
use crate::core::collector::MetricCollector;
use async_trait::async_trait;
use sqlx::PgPool;
use anyhow::Result;

pub struct AvailabilityCollector {
    metrics: AvailabilityMetrics,
}

impl AvailabilityCollector {
    pub fn new() -> Self {
        Self {
            metrics: AvailabilityMetrics::default(),
        }
    }
}

#[async_trait]
impl MetricCollector for AvailabilityCollector {
    fn name(&self) -> &'static str {
        "availability"
    }
    
    async fn collect(&self, pool: &PgPool) -> Result<()> {
        let rows = sqlx::query!(
            r#"
            SELECT
                CASE WHEN channel_code='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(
                    SUM(CASE WHEN status='success' THEN 1 ELSE 0 END)::numeric * 100.0
                    / NULLIF(COUNT(*), 0)
                , 1) as availability
            FROM channel_request_log
            WHERE created_at >= NOW() - INTERVAL '3 hours'
            GROUP BY grp
            "#
        )
        .fetch_all(pool)
        .await?;
        
        for row in rows {
            if let (Some(grp), Some(availability)) = (row.grp, row.availability) {
                self.metrics.percent(&grp).set(availability);
            }
        }
        
        Ok(())
    }
}
```

**运行测试**：`cargo test` → ✅ 通过

#### 🔵 Refactor: 完善 SQL 和错误处理
```rust
async fn collect(&self, pool: &PgPool) -> Result<()> {
    let rows = sqlx::query!(
        r#"
        SELECT
            CASE WHEN channel_code='aws' THEN 'aws' ELSE 'special' END as grp,
            ROUND(
                SUM(CASE WHEN status='success' THEN 1 ELSE 0 END)::numeric * 100.0
                / NULLIF(SUM(CASE WHEN
                    NOT (
                        response_code IN (400,404,413,429)
                        OR (response_code = -1 AND error_message LIKE '%unconfigured%')
                        OR response_code IN (401,403)
                        OR (response_code = 500 AND error_message LIKE '%credit balance%')
                    )
                    OR status='success' THEN 1 ELSE 0 END), 0)
            , 1) as availability
        FROM channel_request_log
        WHERE created_at >= NOW() - INTERVAL '3 hours'
            AND channel_code IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
        GROUP BY grp
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to query availability: {}", e))?;
    
    for row in rows {
        if let (Some(grp), Some(availability)) = (row.grp, row.availability) {
            self.metrics.percent(&grp).set(availability);
        }
    }
    
    Ok(())
}
```

**运行测试**：`cargo test` → ✅ 仍然通过

---

### Iteration 5: Cost 模块（重复 RGR）

#### 🔴 Red → 🟢 Green → 🔵 Refactor
1. 写 Cost 指标测试 → 实现指标 → 重构
2. 写 Cost 采集器测试 → 实现采集器 → 重构

---

### Iteration 6: Cache 模块（重复 RGR）

#### 🔴 Red → 🟢 Green → 🔵 Refactor
1. 写 Cache 指标测试 → 实现指标 → 重构
2. 写 Cache 采集器测试 → 实现采集器 → 重构

---

### Iteration 7: API 端点

#### 🔴 Red: 写失败的测试
```rust
// tests/api_tests.rs

use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn test_metrics_endpoint() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/metrics").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.text().contains("channel_availability_percent"));
}

#[tokio::test]
async fn test_metadata_endpoint() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/metrics-metadata").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let json: Vec<MetricMetadata> = response.json();
    assert!(!json.is_empty());
    assert!(json.iter().any(|m| m.name == "channel_availability_percent"));
}
```

**运行测试**：`cargo test` → ❌ 失败

#### 🟢 Green: 实现 API
```rust
// src/api/metrics.rs

pub async fn metrics_handler(
    Extension(handle): Extension<PrometheusHandle>,
) -> String {
    handle.render()
}
```

```rust
// src/api/metadata.rs

#[utoipa::path(
    get,
    path = "/metrics-metadata",
    responses((status = 200, body = Vec<MetricMetadata>))
)]
pub async fn get_metrics_metadata() -> Json<Vec<MetricMetadata>> {
    // 实现
}
```

**运行测试**：`cargo test` → ✅ 通过

#### 🔵 Refactor: 添加错误处理和文档

---

### Iteration 8: 主程序集成

#### 🔴 Red: 写集成测试
```rust
// tests/integration_tests.rs

#[tokio::test]
async fn test_full_system_integration() {
    // 启动完整系统
    let app = start_app().await;
    
    // 等待采集器运行
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 验证指标被采集
    let response = reqwest::get("http://localhost:8001/metrics").await.unwrap();
    assert!(response.status().is_success());
    
    let body = response.text().await.unwrap();
    assert!(body.contains("channel_availability_percent"));
}
```

**运行测试**：`cargo test` → ❌ 失败

#### 🟢 Green: 实现主程序
```rust
// src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化
    let pool = create_pool(&env::var("DATABASE_URL")?).await?;
    
    // 注册采集器
    let collectors: Vec<Box<dyn MetricCollector>> = vec![
        Box::new(AvailabilityCollector::new()),
        Box::new(CostCollector::new()),
        Box::new(CacheCollector::new()),
    ];
    
    // 启动采集任务
    for collector in collectors {
        let pool = pool.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(collector.interval()));
            loop {
                ticker.tick().await;
                if let Err(e) = collector.collect(&pool).await {
                    tracing::error!("Collect failed: {}", e);
                }
            }
        });
    }
    
    // 启动 HTTP 服务器
    let app = create_router();
    axum::Server::bind(&"0.0.0.0:8001".parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

**运行测试**：`cargo test` → ✅ 通过

#### 🔵 Refactor: 提取函数，改进结构

---

## 📊 RGR 开发时间表

| Iteration | 内容 | 🔴 Red | 🟢 Green | 🔵 Refactor | 总计 |
|-----------|------|--------|----------|-------------|------|
| 1 | 核心抽象 | 30min | 30min | 30min | 1.5h |
| 2 | 数据库连接池 | 30min | 30min | 30min | 1.5h |
| 3 | Availability 指标 | 30min | 1h | 30min | 2h |
| 4 | Availability 采集器 | 1h | 1.5h | 1h | 3.5h |
| 5 | Cost 模块 | 1h | 1.5h | 1h | 3.5h |
| 6 | Cache 模块 | 1h | 1.5h | 1h | 3.5h |
| 7 | API 端点 | 1h | 1.5h | 1h | 3.5h |
| 8 | 主程序集成 | 1h | 1.5h | 1h | 3.5h |
| 9 | Docker 和部署 | 30min | 1h | 30min | 2h |

**总计**：约 25 小时（3-4 个工作日）

---

## 🛠️ 测试工具和辅助

### 测试数据库设置
```rust
// tests/common/mod.rs

use sqlx::PgPool;

pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test_exporter".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to create test pool");
    
    // 运行迁移
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

pub async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query!("TRUNCATE TABLE channel_request_log, balance_transactions")
        .execute(pool)
        .await
        .expect("Failed to cleanup test db");
}
```

### 测试 Fixtures
```rust
// tests/fixtures/mod.rs

pub fn create_test_availability_data() -> Vec<ChannelRequestLog> {
    vec![
        ChannelRequestLog {
            channel_code: "aws".to_string(),
            status: "success".to_string(),
            response_code: 200,
            created_at: Utc::now() - Duration::hours(1),
        },
        // ...
    ]
}
```

---

## ✅ RGR 最佳实践

### 1. 测试先行
- ❌ 不要先写实现代码
- ✅ 先写测试，确保测试失败
- ✅ 测试描述期望行为

### 2. 最小实现
- ❌ 不要过度设计
- ✅ 只写让测试通过的代码
- ✅ 避免"可能需要"的功能

### 3. 持续重构
- ❌ 不要积累技术债
- ✅ 每次 Green 后立即 Refactor
- ✅ 保持测试通过

### 4. 快速反馈
- ✅ 每个循环 < 10 分钟
- ✅ 频繁运行测试
- ✅ 使用 `cargo watch -x test`

### 5. 测试覆盖率
- ✅ 目标：> 80% 覆盖率
- ✅ 使用 `cargo tarpaulin`
- ✅ 关注关键路径

---

## 🚀 开始 RGR 开发

### 准备工作
```bash
# 1. 创建项目
cargo new rust-exporter
cd rust-exporter

# 2. 添加测试依赖
cargo add --dev tokio-test
cargo add --dev axum-test
cargo add --dev sqlx --features postgres,runtime-tokio-rustls

# 3. 设置测试数据库
createdb test_exporter

# 4. 启动 watch 模式
cargo watch -x test
```

### 第一个 RGR 循环
```bash
# 🔴 Red: 写测试
vim tests/core_collector_tests.rs

# 运行测试（应该失败）
cargo test

# 🟢 Green: 最小实现
vim src/core/collector.rs

# 运行测试（应该通过）
cargo test

# 🔵 Refactor: 改进代码
vim src/core/collector.rs

# 运行测试（仍然通过）
cargo test
```

准备好开始第一个 RGR 循环了吗？
