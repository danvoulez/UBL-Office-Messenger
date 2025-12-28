//! # UBL Runner Core
//!
//! **Title:** SPEC-UBL-RUNNER v1.0  
//! **Status:** NORMATIVE  
//! **Change-Control:** STRICT (no retroactive changes)  
//! **Hash:** BLAKE3 | **Signature:** Ed25519  
//! **Freeze-Date:** 2025-12-25  
//! **Governed by:** SPEC-UBL-CORE v1.0, SPEC-UBL-RUNNER v1.0  
//!
//! Isolated Execution & Receipt Specification
//! Materializes external effects and produces verifiable receipts

#![deny(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors from runner operations
#[derive(Error, Debug, Clone)]
pub enum RunnerError {
    /// Invalid trigger (link not yet committed)
    #[error("Invalid trigger: {0}")]
    InvalidTrigger(String),

    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Artifact violation
    #[error("Artifact violation: {0}")]
    ArtifactViolation(String),

    /// Receipt commit failed
    #[error("Receipt commit failed: {0}")]
    ReceiptCommitFailed(String),

    /// Timeout
    #[error("Execution timeout")]
    Timeout,
}

/// Result type for runner operations
pub type Result<T> = std::result::Result<T, RunnerError>;

/// Execution status (SPEC-UBL-RUNNER v1.0 §7)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Execution succeeded
    Success,
    /// Execution failed
    Failure,
}

/// Artifact produced by execution (SPEC-UBL-RUNNER v1.0 §8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact identifier
    pub artifact_id: String,
    
    /// Type of artifact (e.g., "binary", "log", "output")
    pub artifact_type: String,
    
    /// Size in bytes
    pub size: u64,
    
    /// Content hash (BLAKE3)
    pub content_hash: String,
    
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Execution receipt (SPEC-UBL-RUNNER v1.0 §7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    /// Container that owns this execution
    pub container_id: String,
    
    /// Hash of the link that triggered execution
    pub trigger_link_hash: String,
    
    /// Unique execution ID
    pub execution_id: String,
    
    /// Execution status
    pub status: ExecutionStatus,
    
    /// Artifacts produced
    pub artifacts: Vec<Artifact>,
    
    /// Optional: hash of stdout
    pub stdout_hash: Option<String>,
    
    /// Optional: hash of stderr
    pub stderr_hash: Option<String>,
    
    /// Start timestamp (Unix ns)
    pub started_at: u128,
    
    /// Finish timestamp (Unix ns)
    pub finished_at: u128,
}

impl ExecutionReceipt {
    /// Create a new receipt
    pub fn new(
        container_id: String,
        trigger_link_hash: String,
        execution_id: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        Self {
            container_id,
            trigger_link_hash,
            execution_id,
            status: ExecutionStatus::Success,
            artifacts: Vec::new(),
            stdout_hash: None,
            stderr_hash: None,
            started_at: now,
            finished_at: now,
        }
    }

    /// Add an artifact
    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Set stdout hash
    pub fn set_stdout_hash(&mut self, hash: String) {
        self.stdout_hash = Some(hash);
    }

    /// Set stderr hash
    pub fn set_stderr_hash(&mut self, hash: String) {
        self.stderr_hash = Some(hash);
    }

    /// Mark as failed
    pub fn mark_failed(&mut self) {
        self.status = ExecutionStatus::Failure;
    }

    /// Finish execution
    pub fn finish(&mut self) {
        self.finished_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> u128 {
        (self.finished_at - self.started_at) / 1_000_000
    }
}

/// Job in the execution queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionJob {
    /// Job ID
    pub job_id: String,
    
    /// Container ID
    pub container_id: String,
    
    /// Link that triggered this job
    pub trigger_link_hash: String,
    
    /// Job type (e.g., "build", "test", "deploy")
    pub job_type: String,
    
    /// Payload for execution
    pub payload: HashMap<String, serde_json::Value>,
    
    /// Priority (higher = more urgent)
    pub priority: i32,
    
    /// Created timestamp
    pub created_at: i64,
    
    /// Retry count
    pub retries: u32,
}

impl ExecutionJob {
    /// Create a new job
    pub fn new(
        container_id: String,
        trigger_link_hash: String,
        job_type: String,
    ) -> Self {
        let job_id = format!(
            "job_{}_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            rand::random::<u32>()
        );
        
        Self {
            job_id,
            container_id,
            trigger_link_hash,
            job_type,
            payload: HashMap::new(),
            priority: 0,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            retries: 0,
        }
    }

    /// Add payload item
    pub fn add_payload(&mut self, key: String, value: serde_json::Value) {
        self.payload.insert(key, value);
    }

    /// Increment retry count
    pub fn retry(&mut self) {
        self.retries += 1;
    }
}

/// Runner queue - manages execution jobs
pub struct RunnerQueue {
    jobs: Vec<ExecutionJob>,
    max_retries: u32,
}

impl RunnerQueue {
    /// Create a new queue
    pub fn new(max_retries: u32) -> Self {
        Self {
            jobs: Vec::new(),
            max_retries,
        }
    }

    /// Enqueue a job
    pub fn enqueue(&mut self, job: ExecutionJob) {
        self.jobs.push(job);
        // Sort by priority (higher first)
        self.jobs.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Dequeue next job (pull model)
    pub fn dequeue(&mut self) -> Option<ExecutionJob> {
        if self.jobs.is_empty() {
            return None;
        }
        Some(self.jobs.remove(0))
    }

    /// Requeue a failed job (with retry limit)
    pub fn requeue(&mut self, mut job: ExecutionJob) -> bool {
        if job.retries >= self.max_retries {
            return false; // Max retries exceeded
        }
        job.retry();
        self.enqueue(job);
        true
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

/// Sandbox configuration (SPEC-UBL-RUNNER v1.0 §5)
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Max execution time (seconds)
    pub timeout_secs: u64,
    
    /// Max memory (bytes)
    pub max_memory: u64,
    
    /// Max CPU cores
    pub max_cpu: f32,
    
    /// Network isolation
    pub network_isolated: bool,
    
    /// Filesystem isolation
    pub filesystem_isolated: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 300, // 5 minutes
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_cpu: 1.0,
            network_isolated: true,
            filesystem_isolated: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let receipt = ExecutionReceipt::new(
            "test".to_string(),
            "link_abc".to_string(),
            "exec_123".to_string(),
        );

        assert_eq!(receipt.container_id, "test");
        assert_eq!(receipt.status, ExecutionStatus::Success);
        assert!(receipt.artifacts.is_empty());
    }

    #[test]
    fn test_receipt_artifacts() {
        let mut receipt = ExecutionReceipt::new(
            "test".to_string(),
            "link_abc".to_string(),
            "exec_123".to_string(),
        );

        receipt.add_artifact(Artifact {
            artifact_id: "art_1".to_string(),
            artifact_type: "binary".to_string(),
            size: 1024,
            content_hash: "abc123".to_string(),
            metadata: None,
        });

        assert_eq!(receipt.artifacts.len(), 1);
    }

    #[test]
    fn test_queue_enqueue_dequeue() {
        let mut queue = RunnerQueue::new(3);
        
        let job = ExecutionJob::new(
            "test".to_string(),
            "link_abc".to_string(),
            "build".to_string(),
        );
        
        queue.enqueue(job);
        assert_eq!(queue.len(), 1);
        
        let dequeued = queue.dequeue();
        assert!(dequeued.is_some());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_priority() {
        let mut queue = RunnerQueue::new(3);
        
        let mut low_priority = ExecutionJob::new(
            "test".to_string(),
            "link1".to_string(),
            "build".to_string(),
        );
        low_priority.priority = 1;
        
        let mut high_priority = ExecutionJob::new(
            "test".to_string(),
            "link2".to_string(),
            "build".to_string(),
        );
        high_priority.priority = 10;
        
        queue.enqueue(low_priority);
        queue.enqueue(high_priority);
        
        let first = queue.dequeue().unwrap();
        assert_eq!(first.priority, 10);
    }

    #[test]
    fn test_queue_retry_limit() {
        let mut queue = RunnerQueue::new(2);
        
        let mut job = ExecutionJob::new(
            "test".to_string(),
            "link_abc".to_string(),
            "build".to_string(),
        );
        
        // First requeue - should succeed
        assert!(queue.requeue(job.clone()));
        
        // Simulate another failure
        job.retry();
        
        // Second requeue - should succeed
        assert!(queue.requeue(job.clone()));
        
        // Simulate another failure
        job.retry();
        
        // Third requeue - should fail (max retries exceeded)
        assert!(!queue.requeue(job));
    }
}