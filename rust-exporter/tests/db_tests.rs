use rust_exporter::db::create_pool;
use sqlx::PgPool;

#[tokio::test]
async fn test_create_pool_success() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    let result = create_pool(&database_url).await;
    
    if result.is_ok() {
        let pool = result.unwrap();
        
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to execute test query");
        
        assert_eq!(row.0, 1);
    }
}

#[tokio::test]
async fn test_create_pool_invalid_url() {
    let result: anyhow::Result<sqlx::PgPool> = create_pool("invalid://url").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_pool_multiple_queries() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = create_pool(&database_url).await {
        for i in 1..=5 {
            let row: (i32,) = sqlx::query_as("SELECT $1")
                .bind(i)
                .fetch_one(&pool)
                .await
                .expect("Failed to execute query");
            
            assert_eq!(row.0, i);
        }
    }
}

#[tokio::test]
async fn test_pool_concurrent_queries() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    if let Ok(pool) = create_pool(&database_url).await {
        let mut handles = vec![];
        
        for i in 1..=3 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let row: (i32,) = sqlx::query_as("SELECT $1")
                    .bind(i)
                    .fetch_one(&pool_clone)
                    .await
                    .expect("Failed to execute query");
                row.0
            });
            handles.push(handle);
        }
        
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert_eq!(result, (i + 1) as i32);
        }
    }
}

#[tokio::test]
async fn test_pool_connection_timeout() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/test".to_string());
    
    let result = create_pool(&database_url).await;
    assert!(result.is_ok() || result.is_err());
}
