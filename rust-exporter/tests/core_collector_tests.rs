use async_trait::async_trait;
use rust_exporter::core::collector::MetricCollector;
use sqlx::PgPool;

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

#[tokio::test]
async fn test_collector_trait_basic() {
    let collector = DummyCollector;
    
    assert_eq!(collector.name(), "dummy");
    assert_eq!(collector.interval(), 60);
    assert!(collector.enabled());
}

#[tokio::test]
async fn test_collector_trait_collect() {
    let collector = DummyCollector;
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    let pool = sqlx::PgPool::connect(&database_url).await;
    
    if let Ok(pool) = pool {
        let result = collector.collect(&pool).await;
        assert!(result.is_ok());
    }
}

struct CustomIntervalCollector;

#[async_trait]
impl MetricCollector for CustomIntervalCollector {
    fn name(&self) -> &'static str {
        "custom"
    }
    
    async fn collect(&self, _pool: &PgPool) -> anyhow::Result<()> {
        Ok(())
    }
    
    fn interval(&self) -> u64 {
        30
    }
}

#[tokio::test]
async fn test_collector_custom_interval() {
    let collector = CustomIntervalCollector;
    
    assert_eq!(collector.name(), "custom");
    assert_eq!(collector.interval(), 30);
    assert!(collector.enabled());
}

struct DisabledCollector;

#[async_trait]
impl MetricCollector for DisabledCollector {
    fn name(&self) -> &'static str {
        "disabled"
    }
    
    async fn collect(&self, _pool: &PgPool) -> anyhow::Result<()> {
        Ok(())
    }
    
    fn enabled(&self) -> bool {
        false
    }
}

#[tokio::test]
async fn test_collector_disabled() {
    let collector = DisabledCollector;
    
    assert_eq!(collector.name(), "disabled");
    assert!(!collector.enabled());
}

struct ErrorCollector;

#[async_trait]
impl MetricCollector for ErrorCollector {
    fn name(&self) -> &'static str {
        "error"
    }
    
    async fn collect(&self, _pool: &PgPool) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Test error"))
    }
}

#[tokio::test]
async fn test_collector_error_handling() {
    let collector = ErrorCollector;
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = sqlx::PgPool::connect(&database_url).await {
        let result = collector.collect(&pool).await;
        assert!(result.is_err());
    }
}
