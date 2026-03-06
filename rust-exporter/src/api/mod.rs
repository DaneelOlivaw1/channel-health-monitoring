use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
    extract::State,
};
use metrics_exporter_prometheus::PrometheusHandle;

pub fn create_router(prometheus_handle: PrometheusHandle) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(prometheus_handle)
}

async fn metrics_handler(State(handle): State<PrometheusHandle>) -> impl IntoResponse {
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
