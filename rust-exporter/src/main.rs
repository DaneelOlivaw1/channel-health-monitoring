use anyhow::Result;
use rust_exporter::{
    api::create_router,
    core::collector::MetricCollector,
    db::create_pool,
    metrics::{
        availability::collector::AvailabilityCollector,
        cache::collector::CacheCollector,
        cost::collector::CostCollector,
    },
    pushgateway::{PushgatewayConfig, start_pushgateway_worker},
};
use std::time::Duration;
use tokio::{signal, time};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting Rust Exporter...");
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = create_pool(&database_url).await?;
    info!("Database connection pool created");
    
    let collectors: Vec<Box<dyn MetricCollector>> = vec![
        Box::new(AvailabilityCollector::new()),
        Box::new(CacheCollector::new()),
        Box::new(CostCollector::new()),
    ];
    
    info!("Registered {} collectors", collectors.len());
    
    for collector in collectors.iter() {
        let pool_clone = pool.clone();
        let interval = collector.interval();
        let name = collector.name().to_string();
        
        tokio::spawn(async move {
            let mut interval_timer = time::interval(Duration::from_secs(interval));
            loop {
                interval_timer.tick().await;
                
                let collector_instance: Box<dyn MetricCollector> = match name.as_str() {
                    "availability" => Box::new(AvailabilityCollector::new()),
                    "cache" => Box::new(CacheCollector::new()),
                    "cost" => Box::new(CostCollector::new()),
                    _ => continue,
                };
                
                match collector_instance.collect(&pool_clone).await {
                    Ok(_) => info!("Collected metrics from {}", name),
                    Err(e) => error!("Failed to collect metrics from {}: {}", name, e),
                }
            }
        });
    }
    
    // Start Pushgateway worker if configured
    let pushgateway_config = PushgatewayConfig::from_env()?;
    if pushgateway_config.is_enabled() {
        info!("Pushgateway is enabled, starting worker...");
        tokio::spawn(async move {
            if let Err(e) = start_pushgateway_worker(pushgateway_config).await {
                error!("Pushgateway worker error: {}", e);
            }
        });
    } else {
        info!("Pushgateway is disabled");
    }
    
    let app = create_router();
    
    let addr = "0.0.0.0:8001";
    info!("Starting HTTP server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}
