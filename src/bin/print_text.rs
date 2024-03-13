
use color_eyre::Result;
use tracing_subscriber::EnvFilter;
use bleasy::{Scanner, ScanConfig};
use futures::{FutureExt, StreamExt};
use paris::{info, success, warn};


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().pretty().with_env_filter(EnvFilter::from_default_env()).try_init()?;
    let config = ScanConfig::default();
    let mut scanner = Scanner::new();
    scanner.start(config).await?;
    let mut dev_stream = scanner.device_stream();
    let timeout = std::time::Duration::from_secs(60);
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if let Some(dev) = dev_stream.next().await {
            info!("Device info:");
            info!("Address: {}", dev.address());
            let mut local_name = "NONE";
            if let Some(n) = dev.local_name() {
                local_name = n
            }
            info!("Local name: {}", local_name);
            info!("Characteristics: {:?}", dev.characteristics().await.map(|x| x.len().to_string()).unwrap_or("NONE".to_string()));
            info!("Services: {:?}", dev.service_count().await);
        }
    }
    Ok(())
}