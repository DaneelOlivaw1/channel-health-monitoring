use metrics_exporter_prometheus::PrometheusBuilder;
use rust_exporter::{
    api::create_router,
    core::collector::MetricCollector,
    db::create_pool,
    metrics::{
        availability::collector::AvailabilityCollector,
        cache::collector::CacheCollector,
        cost::collector::CostCollector,
    },
};

#[tokio::test]
async fn test_main_integration() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());

    if let Ok(pool) = create_pool(&database_url).await {
        let collectors: Vec<Box<dyn MetricCollector>> = vec![
            Box::new(AvailabilityCollector::new()),
            Box::new(CacheCollector::new()),
            Box::new(CostCollector::new()),
        ];

        assert_eq!(collectors.len(), 3);
        assert_eq!(collectors[0].name(), "availability");
        assert_eq!(collectors[1].name(), "cache");
        assert_eq!(collectors[2].name(), "cost");
    }
}

#[tokio::test]
async fn test_router_creation() {
    let handle = PrometheusBuilder::new().build_recorder().handle();
    let _router = create_router(handle);
}
