use rust_exporter::metrics::availability::collector::AvailabilityCollector;
use rust_exporter::core::collector::MetricCollector;

#[tokio::test]
async fn test_availability_collector_basic() {
    let collector = AvailabilityCollector::new();
    
    assert_eq!(collector.name(), "availability");
    assert_eq!(collector.interval(), 60);
    assert!(collector.enabled());
}

#[tokio::test]
async fn test_availability_collector_collect() {
    let collector = AvailabilityCollector::new();
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = sqlx::PgPool::connect(&database_url).await {
        let result = collector.collect(&pool).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_availability_collector_multiple_calls() {
    let collector = AvailabilityCollector::new();
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = sqlx::PgPool::connect(&database_url).await {
        for _ in 0..3 {
            let result = collector.collect(&pool).await;
            assert!(result.is_ok());
        }
    }
}
