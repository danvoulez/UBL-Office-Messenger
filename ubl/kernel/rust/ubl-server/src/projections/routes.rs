//! HTTP API routes for projections
//!
//! These are read-only query endpoints that hit projection tables.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::{JobsProjection, MessagesProjection};
use super::jobs::{Job, Approval};
use super::messages::Message;

/// Shared state for projection routes
#[derive(Clone)]
pub struct ProjectionState {
    pub pool: PgPool,
}

/// Query params for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub before_seq: Option<i64>,
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: T,
}

/// Create projection router
pub fn projection_router() -> Router<ProjectionState> {
    Router::new()
        // Jobs
        .route("/jobs", get(list_jobs))
        .route("/jobs/:job_id", get(get_job))
        .route("/jobs/:job_id/approvals", get(get_job_approvals))
        .route("/conversations/:conversation_id/jobs", get(get_conversation_jobs))
        // Messages
        .route("/conversations/:conversation_id/messages", get(get_conversation_messages))
}

/// GET /query/jobs — List all jobs (paginated)
async fn list_jobs(
    State(state): State<ProjectionState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<Job>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(100);
    let before_seq = query.before_seq.unwrap_or(i64::MAX);

    let jobs = sqlx::query_as!(
        Job,
        r#"
        SELECT job_id, conversation_id, title, description, status, priority,
               assigned_to, created_by, created_at, started_at, completed_at,
               cancelled_at, progress, progress_message, result_summary,
               result_artifacts, estimated_duration_seconds,
               estimated_value as "estimated_value: f64",
               last_event_hash, last_event_seq
        FROM projection_jobs
        WHERE last_event_seq < $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
        before_seq,
        limit
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: jobs }))
}

/// GET /query/jobs/:job_id — Get single job
async fn get_job(
    State(state): State<ProjectionState>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<Job>>, (StatusCode, String)> {
    let projection = JobsProjection::new(state.pool);
    
    let job = projection
        .get_job(&job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Job not found".to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: job }))
}

/// GET /query/jobs/:job_id/approvals — Get pending approvals for job
async fn get_job_approvals(
    State(state): State<ProjectionState>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Approval>>>, (StatusCode, String)> {
    let projection = JobsProjection::new(state.pool);
    
    let approvals = projection
        .get_pending_approvals(&job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: approvals }))
}

/// GET /query/conversations/:conversation_id/jobs — Jobs in conversation
async fn get_conversation_jobs(
    State(state): State<ProjectionState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Job>>>, (StatusCode, String)> {
    let projection = JobsProjection::new(state.pool);
    
    let jobs = projection
        .get_jobs_by_conversation(&conversation_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: jobs }))
}

/// GET /query/conversations/:conversation_id/messages — Messages in conversation
async fn get_conversation_messages(
    State(state): State<ProjectionState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<Message>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(100);
    let projection = MessagesProjection::new(state.pool);
    
    let messages = projection
        .get_messages_by_conversation(&conversation_id, limit, query.before_seq)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: messages }))
}

