use rust_exporter::metrics::cache::collector::CacheCollector;
use rust_exporter::core::collector::MetricCollector;

#[tokio::test]
async fn test_cache_collector_basic() {
    let collector = CacheCollector::new();
    
    assert_eq!(collector.name(), "cache");
    assert_eq!(collector.interval(), 60);
    assert!(collector.enabled());
}

#[tokio::test]
async fn test_cache_collector_collect() {
    let collector = CacheCollector::new();
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = sqlx::PgPool::connect(&database_url).await {
        let result = collector.collect(&pool).await;
        assert!(result.is_ok());
    }
}
