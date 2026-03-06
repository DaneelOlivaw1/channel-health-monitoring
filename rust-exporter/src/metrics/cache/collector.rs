use crate::core::collector::MetricCollector;
use async_trait::async_trait;
use sqlx::{PgPool, FromRow};
use anyhow::Result;
use metrics::gauge;

#[derive(FromRow)]
struct CacheRow {
    grp: String,
    reuse_percent: rust_decimal::Decimal,
}

pub struct CacheCollector;

impl CacheCollector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MetricCollector for CacheCollector {
    fn name(&self) -> &'static str {
        "cache"
    }
    
    async fn collect(&self, pool: &PgPool) -> Result<()> {
        let rows = sqlx::query_as::<_, CacheRow>(
            r#"
            SELECT
                CASE WHEN channel_code='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(
                    SUM(CASE WHEN cache_action='read' THEN 1 ELSE 0 END)::numeric * 100.0
                    / NULLIF(COUNT(*), 0)
                , 1) as reuse_percent
            FROM channel_request_log
            WHERE created_at >= NOW() - INTERVAL '3 hours'
                AND channel_code IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
                AND cache_action IN ('read', 'create')
            GROUP BY grp
            "#
        )
        .fetch_all(pool)
        .await?;
        
        for row in rows {
            let reuse_str = row.reuse_percent.to_string();
            let reuse_value: f64 = reuse_str.parse().unwrap_or(0.0);
            gauge!("channel_cache_reuse_percent", "channel_group" => row.grp)
                .set(reuse_value);
        }
        
        Ok(())
    }
}
