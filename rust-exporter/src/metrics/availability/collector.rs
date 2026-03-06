use crate::core::collector::MetricCollector;
use async_trait::async_trait;
use sqlx::{PgPool, FromRow};
use anyhow::Result;
use metrics::gauge;

#[derive(FromRow)]
struct AvailabilityRow {
    grp: String,
    availability: rust_decimal::Decimal,
}

pub struct AvailabilityCollector;

impl AvailabilityCollector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MetricCollector for AvailabilityCollector {
    fn name(&self) -> &'static str {
        "availability"
    }
    
    async fn collect(&self, pool: &PgPool) -> Result<()> {
        let rows = sqlx::query_as::<_, AvailabilityRow>(
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
        )
        .fetch_all(pool)
        .await?;
        
        for row in rows {
            let availability_str = row.availability.to_string();
            let availability_value: f64 = availability_str.parse().unwrap_or(0.0);
            gauge!("channel_availability_percent", "channel_group" => row.grp)
                .set(availability_value);
        }
        
        Ok(())
    }
}
