use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::{Result, Context};
use std::time::Duration;

/// Create a PostgreSQL connection pool
/// 
/// # Arguments
/// 
/// * `database_url` - PostgreSQL connection URL
/// 
/// # Example
/// 
/// ```no_run
/// use rust_exporter::db::create_pool;
/// 
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let pool = create_pool("postgres://localhost/mydb").await?;
///     Ok(())
/// }
/// ```
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .context("Failed to create database connection pool")?;
    
    Ok(pool)
}
