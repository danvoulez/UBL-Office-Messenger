//! Common test utilities and helpers
//! ═══════════════════════════════════════════════════════════════════════════
//! UBL 3.0 Test Infrastructure
//! ═══════════════════════════════════════════════════════════════════════════

use anyhow::Result;
use integration_tests::{OfficeClient, TestContext, UblClient, setup};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

// ═══════════════════════════════════════════════════════════════════════════
// Setup Functions
// ═══════════════════════════════════════════════════════════════════════════

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

/// Setup test context for chaos monkey tests
pub async fn setup_chaos_env() -> Result<TestContext> {
    setup().await
}

// ═══════════════════════════════════════════════════════════════════════════
// Wait/Polling Utilities
// ═══════════════════════════════════════════════════════════════════════════

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

/// Wait for a job to reach a specific state
pub async fn wait_for_job_state(
    ctx: &TestContext,
    job_id: &str,
    expected_state: &str,
    timeout: Duration,
) -> Result<()> {
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        match ctx.ubl_client.get_job(job_id).await {
            Ok(job) if job.state == expected_state => return Ok(()),
            Ok(job) => {
                println!("  Job state: {} (waiting for {})", job.state, expected_state);
            }
            Err(_) => {}
        }
        sleep(Duration::from_millis(500)).await;
    }
    
    Err(anyhow::anyhow!("Timeout waiting for job {} to reach state {}", job_id, expected_state))
}

// ═══════════════════════════════════════════════════════════════════════════
// Snapshot Utilities
// ═══════════════════════════════════════════════════════════════════════════

/// State snapshot for comparison
#[derive(Debug, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub timestamp: String,
    pub entities: Vec<Value>,
    pub conversations: Vec<Value>,
    pub atom_count: usize,
    pub hash_chain_tip: Option<String>,
}

/// Capture current state snapshot
pub async fn capture_snapshot(ctx: &TestContext) -> Result<StateSnapshot> {
    let boot = ctx.ubl_client.bootstrap("T.Snapshot").await?;
    
    Ok(StateSnapshot {
        timestamp: chrono::Utc::now().to_rfc3339(),
        entities: boot.entities,
        conversations: boot.conversations,
        atom_count: 0, // Would need separate query
        hash_chain_tip: None,
    })
}

/// Save snapshot to file for golden comparison
pub fn save_golden_snapshot(name: &str, snapshot: &StateSnapshot) -> Result<()> {
    let path = format!("snapshots/{}.json", name);
    std::fs::create_dir_all("snapshots")?;
    std::fs::write(&path, serde_json::to_string_pretty(snapshot)?)?;
    println!("  Snapshot saved to {}", path);
    Ok(())
}

/// Compare two snapshots
pub fn compare_snapshots(a: &StateSnapshot, b: &StateSnapshot) -> Vec<String> {
    let mut diffs = Vec::new();
    
    if a.entities.len() != b.entities.len() {
        diffs.push(format!("Entity count: {} vs {}", a.entities.len(), b.entities.len()));
    }
    
    if a.conversations.len() != b.conversations.len() {
        diffs.push(format!("Conversation count: {} vs {}", a.conversations.len(), b.conversations.len()));
    }
    
    if a.atom_count != b.atom_count {
        diffs.push(format!("Atom count: {} vs {}", a.atom_count, b.atom_count));
    }
    
    diffs
}

// ═══════════════════════════════════════════════════════════════════════════
// Crypto Validation Utilities
// ═══════════════════════════════════════════════════════════════════════════

/// Validate hash format (BLAKE3/SHA256 = 64 hex chars)
pub fn validate_hash_format(hash: &str) -> Result<()> {
    if hash.len() != 64 {
        return Err(anyhow::anyhow!("Hash length {} != 64", hash.len()));
    }
    
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow::anyhow!("Hash contains non-hex characters"));
    }
    
    if hash.chars().all(|c| c == '0') {
        return Err(anyhow::anyhow!("Hash is all zeros (invalid)"));
    }
    
    Ok(())
}

/// Check hash chain integrity in a list of items
pub fn validate_hash_chain(items: &[Value]) -> Result<()> {
    let mut seen_hashes = std::collections::HashSet::new();
    let mut prev_seq = 0i64;
    
    for item in items {
        if let Some(hash) = item["hash"].as_str() {
            // Check uniqueness
            if !seen_hashes.insert(hash.to_string()) {
                return Err(anyhow::anyhow!("Duplicate hash found: {}", hash));
            }
            
            // Check format
            validate_hash_format(hash)?;
        }
        
        // Check sequence monotonicity
        if let Some(seq) = item["sequence"].as_i64() {
            if seq <= prev_seq && prev_seq != 0 {
                return Err(anyhow::anyhow!("Sequence not monotonic: {} after {}", seq, prev_seq));
            }
            prev_seq = seq;
        }
    }
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Performance Utilities
// ═══════════════════════════════════════════════════════════════════════════

/// Collect timing samples for a function
pub async fn collect_timing_samples<F, Fut, T>(
    mut f: F,
    samples: usize,
) -> Vec<Duration>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut durations = Vec::with_capacity(samples);
    
    for _ in 0..samples {
        let start = std::time::Instant::now();
        let _ = f().await;
        durations.push(start.elapsed());
    }
    
    durations.sort();
    durations
}

/// Calculate percentile from sorted durations
pub fn percentile(sorted: &[Duration], p: usize) -> Duration {
    let idx = (sorted.len() * p / 100).min(sorted.len() - 1);
    sorted[idx]
}

/// Performance statistics
#[derive(Debug)]
pub struct PerfStats {
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub min: Duration,
}

impl PerfStats {
    pub fn from_samples(mut samples: Vec<Duration>) -> Self {
        samples.sort();
        let len = samples.len();
        
        Self {
            p50: samples[len * 50 / 100],
            p95: samples[len * 95 / 100],
            p99: samples[len * 99 / 100],
            max: *samples.last().unwrap_or(&Duration::ZERO),
            min: *samples.first().unwrap_or(&Duration::ZERO),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Re-exports
// ═══════════════════════════════════════════════════════════════════════════

// Re-export commonly used types (avoid duplicate re-exports)
pub type OfficeClient = integration_tests::OfficeClient;
pub type TestContext = integration_tests::TestContext;
pub type UblClient = integration_tests::UblClient;

// Re-export request/response types
pub use integration_tests::{
    SendMessageRequest, 
    SendMessageResponse,
    JobActionRequest,
    JobActionResponse,
    TimelineResponse,
    JobResponse,
};

/// Chaos test context (alias for TestContext)
pub type ChaosTestContext = TestContext;

