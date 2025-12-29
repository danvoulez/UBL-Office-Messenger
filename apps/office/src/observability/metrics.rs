//! # Office Runtime Metrics
//!
//! Prometheus metrics for Office Runtime operations.

use lazy_static::lazy_static;
use prometheus::{IntCounterVec, IntGaugeVec, HistogramVec, register_int_counter_vec, register_int_gauge_vec, register_histogram_vec};

lazy_static! {
    /// Total entity operations by type and status
    pub static ref ENTITY_OPS: IntCounterVec = register_int_counter_vec!(
        "office_entity_operations_total",
        "Total entity operations by type and status",
        &["operation", "status"]
    ).unwrap();
    
    /// Active entities count
    pub static ref ACTIVE_ENTITIES: IntGaugeVec = register_int_gauge_vec!(
        "office_active_entities",
        "Number of active entities",
        &["entity_type"]
    ).unwrap();
    
    /// Job operations by type and status
    pub static ref JOB_OPS: IntCounterVec = register_int_counter_vec!(
        "office_job_operations_total",
        "Total job operations by type and status",
        &["operation", "status"]
    ).unwrap();
    
    /// LLM API calls by provider and status
    pub static ref LLM_CALLS: IntCounterVec = register_int_counter_vec!(
        "office_llm_calls_total",
        "Total LLM API calls by provider and status",
        &["provider", "status"]
    ).unwrap();
    
    /// LLM API call latency
    pub static ref LLM_LATENCY: HistogramVec = register_histogram_vec!(
        "office_llm_call_duration_seconds",
        "LLM API call duration in seconds",
        &["provider", "operation"]
    ).unwrap();
    
    /// Context frame building operations
    pub static ref CONTEXT_OPS: IntCounterVec = register_int_counter_vec!(
        "office_context_operations_total",
        "Total context frame building operations",
        &["operation", "status"]
    ).unwrap();
    
    /// Context frame building latency
    pub static ref CONTEXT_LATENCY: HistogramVec = register_histogram_vec!(
        "office_context_build_duration_seconds",
        "Context frame building duration in seconds",
        &["entity_id"]
    ).unwrap();
    
    /// UBL commit operations
    pub static ref UBL_COMMITS: IntCounterVec = register_int_counter_vec!(
        "office_ubl_commits_total",
        "Total UBL commit operations",
        &["status"]
    ).unwrap();
    
    /// UBL commit latency
    pub static ref UBL_COMMIT_LATENCY: HistogramVec = register_histogram_vec!(
        "office_ubl_commit_duration_seconds",
        "UBL commit duration in seconds",
        &["container_id"]
    ).unwrap();
}

/// Initialize metrics (called on startup)
pub fn init_metrics() {
    // Metrics are automatically registered via lazy_static
    // This function can be used for any additional initialization
}

