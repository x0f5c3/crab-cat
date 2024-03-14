
use color_eyre::Result;
use tracing_subscriber::EnvFilter;
use bleasy::{Scanner, ScanConfig};
use futures::{FutureExt, StreamExt};
use paris::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().pretty().with_env_filter(EnvFilter::from_default_env()).try_init()?;
    let config = ScanConfig::default();
    let mut scanner = Scanner::new();
    scanner.start(config).await?;
    let mut dev_stream = scanner.device_stream();
    let sigint = tokio::signal::ctrl_c();
    let (send, mut recv) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                paris::warn!("Received CTRL-C, stopping search...");
                match send.send(true).await {
                    Err(e) => {
                        error!("Got error while sending stop signal {}", e);
                    }
                    _ => {}
                }
            },
            Err(e) => {
                error!("Got error waiting on ctrl-c, still will close");
                match send.send(true).await {
                    Err(e) => {
                        error!("Got error while sending stop signal {}", e);
                    }
                    _ => {}
                }

            }
        }
    });
    let timeout = std::time::Duration::from_secs(60);
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        match recv.try_recv() {
            Ok(true) => {
                break
            },
            _ => {}
        }
        if let Some(dev) = dev_stream.next().await {
            info!("Device info:");
            info!("Address: {}", dev.address());
            let mut local_name = "NONE";
            if let Some(n) = dev.local_name().await {
                local_name = &*n
            }
            info!("Local name: {}", local_name);
            info!("Characteristics: {:?}", dev.characteristics().await.map(|x| x.len().to_string()).unwrap_or("NONE".to_string()));
            info!("Services: {:?}", dev.service_count().await);
        }
    }
    Ok(())
}