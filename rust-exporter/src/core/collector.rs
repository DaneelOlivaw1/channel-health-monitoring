use async_trait::async_trait;
use sqlx::PgPool;
use anyhow::Result;

#[async_trait]
pub trait MetricCollector: Send + Sync {
    fn name(&self) -> &'static str;
    
    async fn collect(&self, pool: &PgPool) -> Result<()>;
    
    fn interval(&self) -> u64 {
        60
    }
    
    fn enabled(&self) -> bool {
        true
    }
}
