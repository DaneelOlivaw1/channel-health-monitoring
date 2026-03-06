use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use metrics_exporter_prometheus::PrometheusBuilder;
use tower::ServiceExt;

fn test_prometheus_handle() -> metrics_exporter_prometheus::PrometheusHandle {
    PrometheusBuilder::new().build_recorder().handle()
}

#[tokio::test]
async fn test_metrics_endpoint_exists() {
    let app = rust_exporter::api::create_router(test_prometheus_handle());

    let response = app
        .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_endpoint_prometheus_format() {
    let app = rust_exporter::api::create_router(test_prometheus_handle());

    let response = app
        .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/plain"));
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = rust_exporter::api::create_router(test_prometheus_handle());

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
