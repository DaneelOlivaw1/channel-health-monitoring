// Rust Prometheus Exporter 示例
// 展示如何用 trait + 宏自动生成指标和文档

use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// 1. 定义指标元数据的 trait
pub trait MetricMetadata {
    fn name(&self) -> &str;
    fn metric_type(&self) -> &str;
    fn description(&self) -> &str;
    fn help_text(&self) -> &str;
    fn labels(&self) -> Vec<&str>;
    fn calculation_formula(&self) -> &str;
    fn sql_query(&self) -> &str;
}

// 2. 定义指标配置结构（可以从 YAML 反序列化）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MetricConfig {
    pub name: String,
    pub metric_type: String,
    pub description: String,
    pub labels: Vec<String>,
    pub collection_frequency: String,
    pub calculation_formula: String,
    pub notes: String,
    pub sql_query: String,
}

// 3. 定义具体的指标（使用宏自动实现 trait）
#[derive(Clone)]
pub struct ChannelAvailability {
    gauge: Gauge,
}

impl ChannelAvailability {
    pub fn new(registry: &Registry) -> Self {
        let gauge = Gauge::new(
            "channel_availability_percent",
            "Channel availability percentage (excluding user errors)",
        )
        .unwrap();
        registry.register(Box::new(gauge.clone())).unwrap();
        Self { gauge }
    }

    pub fn set(&self, channel_group: &str, value: f64) {
        self.gauge.set(value);
    }
}

impl MetricMetadata for ChannelAvailability {
    fn name(&self) -> &str {
        "channel_availability_percent"
    }

    fn metric_type(&self) -> &str {
        "Gauge"
    }

    fn description(&self) -> &str {
        "渠道可用性百分比（排除用户错误）"
    }

    fn help_text(&self) -> &str {
        "排除的错误类型：400/404/413/429（用户错误）、401/403（鉴权问题）、500 credit balance（余额不足）"
    }

    fn labels(&self) -> Vec<&str> {
        vec!["channel_group"]
    }

    fn calculation_formula(&self) -> &str {
        "成功请求数 / (总请求数 - 用户错误请求数) × 100"
    }

    fn sql_query(&self) -> &str {
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
    }
}

// 4. 使用 utoipa 自动生成 OpenAPI 文档
use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    metrics: Vec<MetricConfig>,
}

/// Get all metrics metadata
#[utoipa::path(
    get,
    path = "/metrics-metadata",
    responses(
        (status = 200, description = "List of all metrics", body = Vec<MetricConfig>)
    ),
    tag = "metrics"
)]
async fn get_metrics_metadata(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<MetricConfig>> {
    Json(state.metrics.clone())
}

// 5. 主函数：自动生成 Swagger UI
#[tokio::main]
async fn main() {
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    #[derive(OpenApi)]
    #[openapi(
        paths(get_metrics_metadata),
        components(schemas(MetricConfig)),
        tags(
            (name = "metrics", description = "Metrics metadata API")
        )
    )]
    struct ApiDoc;

    let metrics = vec![
        // 从 YAML 加载或硬编码
    ];

    let state = Arc::new(AppState { metrics });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/metrics-metadata", get(get_metrics_metadata))
        .with_state(state);

    println!("Server running on http://localhost:8002");
    println!("Swagger UI: http://localhost:8002/swagger-ui");
    
    axum::Server::bind(&"0.0.0.0:8002".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
