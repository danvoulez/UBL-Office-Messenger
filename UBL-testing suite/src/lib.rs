pub mod helpers;
pub mod clients;
pub mod fixtures;

pub use helpers::*;
pub use clients::*;
pub use fixtures::*;

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

/// Wait for all services to be healthy
pub async fn wait_for_services() -> Result<()> {
    let services = vec![
        ("UBL Kernel", "http://localhost:8080/health"),
        ("Office", "http://localhost:8081/health"),
    ];
    
    for (name, url) in services {
        println!("🔍 Waiting for {} to be healthy...", name);
        
        for attempt in 1..=30 {
            match reqwest::get(url).await {
                Ok(resp) if resp.status().is_success() => {
                    println!("✅ {} is healthy", name);
                    break;
                }
                _ => {
                    if attempt == 30 {
                        anyhow::bail!("{} failed to become healthy", name);
                    }
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }
    
    Ok(())
}

/// Setup test environment
pub async fn setup() -> Result<TestContext> {
    tracing_subscriber::fmt::init();
    
    wait_for_services().await?;
    
    let ubl_client = UblClient::new("http://localhost:8080".to_string());
    let office_client = OfficeClient::new("http://localhost:8081".to_string());
    
    Ok(TestContext {
        ubl_client,
        office_client,
    })
}

/// Test context
pub struct TestContext {
    pub ubl_client: UblClient,
    pub office_client: OfficeClient,
}