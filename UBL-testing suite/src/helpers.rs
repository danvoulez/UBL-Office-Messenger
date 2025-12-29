use anyhow::Result;
use serde_json::Value;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Retry a function until it succeeds or times out
pub async fn retry_until_success<F, Fut, T>(
    mut f: F,
    max_attempts: u32,
    delay:  Duration,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_attempts {
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

/// Poll until a condition is met
pub async fn poll_until<F, Fut>(
    mut condition: F,
    timeout_duration: Duration,
    poll_interval: Duration,
) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let result = timeout(timeout_duration, async {
        loop {
            if condition().await {
                return;
            }
            sleep(poll_interval).await;
        }
    })
    .await;
    
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!("Timeout waiting for condition")),
    }
}

/// Assert JSON contains expected fields
pub fn assert_json_contains(actual: &Value, expected: &Value) -> Result<()> {
    match (actual, expected) {
        (Value::Object(actual_map), Value::Object(expected_map)) => {
            for (key, expected_value) in expected_map {
                let actual_value = actual_map
                    .get(key)
                    .ok_or_else(|| anyhow::anyhow!("Missing key: {}", key))?;
                
                assert_json_contains(actual_value, expected_value)?;
            }
            Ok(())
        }
        (actual, expected) if actual == expected => Ok(()),
        (actual, expected) => {
            Err(anyhow::anyhow!(
                "JSON mismatch: expected {:?}, got {:?}",
                expected,
                actual
            ))
        }
    }
}

/// Generate test ID
pub fn test_id(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::new_v4())
}