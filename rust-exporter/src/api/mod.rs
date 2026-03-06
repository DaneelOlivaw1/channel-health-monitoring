use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
};

pub fn create_router() -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
}

async fn metrics_handler() -> impl IntoResponse {
    let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .build_recorder()
        .handle();
    
    let metrics = handle.render();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        metrics,
    )
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
