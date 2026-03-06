use crate::core::collector::MetricCollector;
use async_trait::async_trait;
use sqlx::{PgPool, FromRow};
use anyhow::Result;
use metrics::gauge;

#[derive(FromRow)]
struct CostRow {
    grp: String,
    opus_price: Option<rust_decimal::Decimal>,
    sonnet_price: Option<rust_decimal::Decimal>,
    avg_price: rust_decimal::Decimal,
}

pub struct CostCollector;

impl CostCollector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MetricCollector for CostCollector {
    fn name(&self) -> &'static str {
        "cost"
    }
    
    async fn collect(&self, pool: &PgPool) -> Result<()> {
        let rows = sqlx::query_as::<_, CostRow>(
            r#"
            SELECT
                CASE WHEN channel_used='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(AVG(CASE WHEN model_name LIKE '%opus%' THEN final_price_cny END)::numeric, 2) as opus_price,
                ROUND(AVG(CASE WHEN model_name LIKE '%sonnet%' THEN final_price_cny END)::numeric, 2) as sonnet_price,
                ROUND(AVG(final_price_cny)::numeric, 2) as avg_price
            FROM balance_transactions
            WHERE created_at >= NOW() - INTERVAL '3 hours'
                AND type='consume' AND transaction_status='completed'
                AND channel_used IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
            GROUP BY grp
            "#
        )
        .fetch_all(pool)
        .await?;
        
        for row in rows {
            if let Some(opus) = row.opus_price {
                let opus_str = opus.to_string();
                let opus_value: f64 = opus_str.parse().unwrap_or(0.0);
                gauge!("channel_avg_cost_cny_opus", "channel_group" => row.grp.clone())
                    .set(opus_value);
            }
            if let Some(sonnet) = row.sonnet_price {
                let sonnet_str = sonnet.to_string();
                let sonnet_value: f64 = sonnet_str.parse().unwrap_or(0.0);
                gauge!("channel_avg_cost_cny_sonnet", "channel_group" => row.grp.clone())
                    .set(sonnet_value);
            }
            let avg_str = row.avg_price.to_string();
            let avg_value: f64 = avg_str.parse().unwrap_or(0.0);
            gauge!("channel_avg_cost_cny_all", "channel_group" => row.grp)
                .set(avg_value);
        }
        
        Ok(())
    }
}
