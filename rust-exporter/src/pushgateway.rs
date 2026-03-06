use anyhow::Result;
use std::time::Duration;
use tokio::time;

pub struct PushgatewayConfig {
    pub url: String,
    pub job_name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub interval: u64,
}

impl PushgatewayConfig {
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("PUSHGATEWAY_URL")
            .unwrap_or_else(|_| "".to_string());
        
        let job_name = std::env::var("PUSHGATEWAY_JOB")
            .unwrap_or_else(|_| "rust-exporter".to_string());
        
        let username = std::env::var("PUSHGATEWAY_USERNAME").ok();
        let password = std::env::var("PUSHGATEWAY_PASSWORD").ok();
        
        let interval = std::env::var("PUSHGATEWAY_INTERVAL")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60);
        
        Ok(Self {
            url,
            job_name,
            username,
            password,
            interval,
        })
    }
    
    pub fn is_enabled(&self) -> bool {
        !self.url.is_empty()
    }
}

pub async fn start_pushgateway_worker(
    config: PushgatewayConfig,
    handle: metrics_exporter_prometheus::PrometheusHandle,
) -> Result<()> {
    if !config.is_enabled() {
        tracing::info!("Pushgateway is disabled (no URL configured)");
        return Ok(());
    }
    
    tracing::info!(
        "Starting Pushgateway worker: URL={}, Job={}, Interval={}s",
        config.url,
        config.job_name,
        config.interval
    );
    
    let mut interval_timer = time::interval(Duration::from_secs(config.interval));
    
    loop {
        interval_timer.tick().await;
        
        match push_metrics(&config, &handle).await {
            Ok(_) => {
                tracing::info!("Successfully pushed metrics to Pushgateway");
            }
            Err(e) => {
                tracing::error!("Failed to push metrics to Pushgateway: {}", e);
            }
        }
    }
}

async fn push_metrics(
    config: &PushgatewayConfig,
    handle: &metrics_exporter_prometheus::PrometheusHandle,
) -> Result<()> {
    let metrics_text = handle.render();
    
    let client = reqwest::Client::new();
    let mut request = client
        .post(format!(
            "{}/metrics/job/{}",
            config.url.trim_end_matches('/'),
            config.job_name
        ))
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_text);
    
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        request = request.basic_auth(username, Some(password));
    }
    
    let response = request.send().await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Pushgateway returned error {}: {}", status, body);
    }
    
    Ok(())
}
