# Rust Exporter 模块化架构设计

## 🏗️ 架构原则

### 问题
- ❌ 所有指标耦合在一个文件
- ❌ 难以扩展新的指标类别
- ❌ 采集逻辑和指标定义混在一起
- ❌ 无法独立测试各个模块

### 解决方案
- ✅ 按业务领域分类指标（Availability、Cost、Cache 等）
- ✅ 每个类别独立模块，可插拔
- ✅ 统一的 Collector trait
- ✅ 自动注册和发现

---

## 📁 新的项目结构

```
rust-exporter/
├── Cargo.toml
├── src/
│   ├── main.rs                    # 主程序：组装所有模块
│   │
│   ├── core/                      # 核心抽象
│   │   ├── mod.rs
│   │   ├── collector.rs           # Collector trait 定义
│   │   ├── registry.rs            # 指标注册中心
│   │   └── config.rs              # 配置管理
│   │
│   ├── metrics/                   # 指标定义（按类别分类）
│   │   ├── mod.rs
│   │   │
│   │   ├── availability/          # 可用性指标模块
│   │   │   ├── mod.rs
│   │   │   ├── metrics.rs         # #[derive(Metrics)]
│   │   │   └── collector.rs       # 实现 Collector trait
│   │   │
│   │   ├── cost/                  # 成本指标模块
│   │   │   ├── mod.rs
│   │   │   ├── metrics.rs
│   │   │   └── collector.rs
│   │   │
│   │   ├── cache/                 # 缓存指标模块
│   │   │   ├── mod.rs
│   │   │   ├── metrics.rs
│   │   │   └── collector.rs
│   │   │
│   │   └── health/                # 健康检查指标（未来扩展）
│   │       ├── mod.rs
│   │       ├── metrics.rs
│   │       └── collector.rs
│   │
│   ├── api/                       # API 端点
│   │   ├── mod.rs
│   │   ├── metrics.rs             # /metrics endpoint
│   │   ├── metadata.rs            # /metrics-metadata endpoint
│   │   └── docs.rs                # OpenAPI 文档
│   │
│   └── db.rs                      # 数据库连接池
│
└── tests/
    ├── availability_tests.rs
    ├── cost_tests.rs
    └── cache_tests.rs
```

---

## 🎯 核心设计：Collector Trait

### 1. 定义统一接口

```rust
// src/core/collector.rs

use async_trait::async_trait;
use sqlx::PgPool;
use anyhow::Result;

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
        60  // 默认 60 秒
    }
    
    /// 是否启用（可以通过配置控制）
    fn enabled(&self) -> bool {
        true
    }
}

/// 指标元数据（用于 API 文档）
pub trait MetricMetadata {
    fn category(&self) -> &'static str;
    fn metrics(&self) -> Vec<MetricInfo>;
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MetricInfo {
    pub name: String,
    pub metric_type: String,
    pub description: String,
    pub labels: Vec<String>,
}
```

---

## 📊 模块化实现示例

### 2. Availability 模块（独立）

```rust
// src/metrics/availability/metrics.rs

use metrics_derive::Metrics;

/// 可用性指标
#[derive(Metrics, Clone)]
#[metrics(scope = "channel_availability")]
pub struct AvailabilityMetrics {
    /// 渠道可用性百分比（排除用户错误）
    /// 
    /// 计算公式：成功请求数 / (总请求数 - 用户错误请求数) × 100
    /// 排除：400/404/413/429（用户错误）、401/403（鉴权）
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

```rust
// src/metrics/availability/mod.rs

mod metrics;
mod collector;

pub use metrics::AvailabilityMetrics;
pub use collector::AvailabilityCollector;
```

---

### 3. Cost 模块（独立）

```rust
// src/metrics/cost/metrics.rs

use metrics_derive::Metrics;

/// 成本指标
#[derive(Metrics, Clone)]
#[metrics(scope = "channel_cost")]
pub struct CostMetrics {
    /// Opus 模型平均成本（人民币/次）
    #[metric(labels = ["channel_group"])]
    pub opus_cny: Gauge,
    
    /// Sonnet 模型平均成本（人民币/次）
    #[metric(labels = ["channel_group"])]
    pub sonnet_cny: Gauge,
    
    /// 所有模型平均成本（人民币/次）
    #[metric(labels = ["channel_group"])]
    pub all_cny: Gauge,
}
```

```rust
// src/metrics/cost/collector.rs

use super::metrics::CostMetrics;
use crate::core::collector::MetricCollector;
use async_trait::async_trait;
use sqlx::PgPool;
use anyhow::Result;

pub struct CostCollector {
    metrics: CostMetrics,
}

impl CostCollector {
    pub fn new() -> Self {
        Self {
            metrics: CostMetrics::default(),
        }
    }
}

#[async_trait]
impl MetricCollector for CostCollector {
    fn name(&self) -> &'static str {
        "cost"
    }
    
    async fn collect(&self, pool: &PgPool) -> Result<()> {
        let rows = sqlx::query!(
            r#"
            SELECT
                CASE WHEN channel_used='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(AVG(CASE WHEN model_name LIKE '%opus%' THEN final_price_cny END)::numeric, 2) as opus_price,
                ROUND(AVG(CASE WHEN model_name LIKE '%sonnet%' THEN final_price_cny END)::numeric, 2) as sonnet_price,
                ROUND(AVG(final_price_cny)::numeric, 2) as avg_price
            FROM balance_transactions
            WHERE created_at >= NOW() - INTERVAL '3 hours'
                AND type='consume' AND transaction_status='completed'
                AND channel_used IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
            GROUP BY grp
            "#
        )
        .fetch_all(pool)
        .await?;
        
        for row in rows {
            if let Some(grp) = row.grp {
                if let Some(opus) = row.opus_price {
                    self.metrics.opus_cny(&grp).set(opus);
                }
                if let Some(sonnet) = row.sonnet_price {
                    self.metrics.sonnet_cny(&grp).set(sonnet);
                }
                if let Some(avg) = row.avg_price {
                    self.metrics.all_cny(&grp).set(avg);
                }
            }
        }
        
        Ok(())
    }
}
```

---

### 4. 主程序：自动组装

```rust
// src/main.rs

use crate::core::collector::MetricCollector;
use crate::metrics::{
    availability::AvailabilityCollector,
    cost::CostCollector,
    cache::CacheCollector,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 数据库连接池
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    
    // 注册所有采集器（可插拔）
    let collectors: Vec<Box<dyn MetricCollector>> = vec![
        Box::new(AvailabilityCollector::new()),
        Box::new(CostCollector::new()),
        Box::new(CacheCollector::new()),
        // 未来添加新模块，只需在这里加一行
        // Box::new(HealthCollector::new()),
    ];
    
    // 启动定时采集任务
    for collector in collectors {
        let pool = pool.clone();
        let interval = collector.interval();
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(interval));
            loop {
                ticker.tick().await;
                
                if let Err(e) = collector.collect(&pool).await {
                    tracing::error!(
                        collector = collector.name(),
                        error = %e,
                        "Failed to collect metrics"
                    );
                }
            }
        });
    }
    
    // 启动 HTTP 服务器
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/metrics-metadata", get(metadata_handler))
        .merge(SwaggerUi::new("/swagger-ui"));
    
    axum::Server::bind(&"0.0.0.0:8001".parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

---

## 🎯 架构优势

### 1. **高内聚，低耦合**
- 每个指标类别独立模块
- 修改 Availability 不影响 Cost
- 可以独立测试每个模块

### 2. **可插拔**
```rust
// 添加新模块只需 3 步：
// 1. 创建 src/metrics/new_category/
// 2. 实现 MetricCollector trait
// 3. 在 main.rs 注册：Box::new(NewCollector::new())
```

### 3. **类型安全**
```rust
// 编译时检查，不会出现运行时错误
self.metrics.opus_cny("aws").set(0.54);  // ✅ 类型安全
self.metrics.opus_cny("aws").set("0.54"); // ❌ 编译错误
```

### 4. **易于测试**
```rust
#[tokio::test]
async fn test_availability_collector() {
    let collector = AvailabilityCollector::new();
    let pool = create_test_pool().await;
    
    collector.collect(&pool).await.unwrap();
    
    // 验证指标值
}
```

### 5. **配置灵活**
```rust
impl MetricCollector for AvailabilityCollector {
    fn enabled(&self) -> bool {
        env::var("ENABLE_AVAILABILITY_METRICS")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true)
    }
    
    fn interval(&self) -> u64 {
        env::var("AVAILABILITY_INTERVAL")
            .unwrap_or("60".to_string())
            .parse()
            .unwrap_or(60)
    }
}
```

---

## 📊 对比

| 特性 | 单文件架构 | 模块化架构 |
|------|-----------|-----------|
| **可维护性** | ❌ 所有代码混在一起 | ✅ 按类别清晰分离 |
| **可扩展性** | ❌ 添加新指标需改多处 | ✅ 只需添加新模块 |
| **可测试性** | ❌ 难以单独测试 | ✅ 每个模块独立测试 |
| **团队协作** | ❌ 容易冲突 | ✅ 不同人负责不同模块 |
| **性能** | ⚠️ 所有指标同时采集 | ✅ 可配置不同采集间隔 |

---

## 🚀 未来扩展示例

### 添加新的 Health 指标模块

```bash
# 1. 创建新模块
mkdir -p src/metrics/health

# 2. 定义指标
cat > src/metrics/health/metrics.rs << 'EOF'
#[derive(Metrics, Clone)]
#[metrics(scope = "channel_health")]
pub struct HealthMetrics {
    /// 数据库连接池健康状态
    #[metric(labels = ["pool_name"])]
    pub db_pool_health: Gauge,
}
EOF

# 3. 实现采集器
cat > src/metrics/health/collector.rs << 'EOF'
pub struct HealthCollector { /* ... */ }

#[async_trait]
impl MetricCollector for HealthCollector {
    fn name(&self) -> &'static str { "health" }
    async fn collect(&self, pool: &PgPool) -> Result<()> {
        // 采集逻辑
    }
}
EOF

# 4. 在 main.rs 注册
# collectors.push(Box::new(HealthCollector::new()));
```

**完成！** 不需要修改其他任何代码。

---

## 📝 总结

这个架构：
- ✅ **解耦**：每个指标类别独立
- ✅ **可扩展**：添加新模块只需 3 步
- ✅ **类型安全**：编译时检查
- ✅ **易测试**：每个模块独立测试
- ✅ **生产级**：参考 Reth、Vector 等大型项目

准备好用这个架构重写了吗？
