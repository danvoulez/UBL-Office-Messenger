//! Common test utilities and helpers

use anyhow::Result;
use integration_tests::{OfficeClient, TestContext, UblClient, setup};
use std::time::Duration;
use tokio::time::sleep;

/// Setup test context for golden run
pub async fn setup_golden_run() -> Result<TestContext> {
    setup().await
}

/// Setup test context for resilience tests
pub async fn setup_resilience() -> Result<TestContext> {
    setup().await
}

/// Setup test context for failure recovery
pub async fn setup_failure_recovery() -> Result<TestContext> {
    setup().await
}

/// Wait for a condition with timeout
pub async fn wait_for<F, Fut>(mut condition: F, timeout: Duration, interval: Duration) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    use tokio::time::Instant;
    
    let start = Instant::now();
    loop {
        if condition().await {
            return Ok(());
        }
        
        if start.elapsed() >= timeout {
            return Err(anyhow::anyhow!("Timeout waiting for condition"));
        }
        
        sleep(interval).await;
    }
}

// Re-export commonly used types
pub use integration_tests::{OfficeClient, TestContext, UblClient};

