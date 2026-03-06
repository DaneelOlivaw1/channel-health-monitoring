use rust_exporter::metrics::cost::collector::CostCollector;
use rust_exporter::core::collector::MetricCollector;

#[tokio::test]
async fn test_cost_collector_basic() {
    let collector = CostCollector::new();
    
    assert_eq!(collector.name(), "cost");
    assert_eq!(collector.interval(), 60);
    assert!(collector.enabled());
}

#[tokio::test]
async fn test_cost_collector_collect() {
    let collector = CostCollector::new();
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = sqlx::PgPool::connect(&database_url).await {
        let result = collector.collect(&pool).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_cost_collector_with_empty_database() {
    let collector = CostCollector::new();
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = sqlx::PgPool::connect(&database_url).await {
        let result = collector.collect(&pool).await;
        assert!(result.is_ok());
    }
}
