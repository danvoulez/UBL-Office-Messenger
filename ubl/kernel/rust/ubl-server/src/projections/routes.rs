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

use super::{JobsProjection, MessagesProjection, OfficeProjection};
use super::jobs::{Job, Approval};
use super::messages::Message;
use super::office::{EntityRow, SessionRow, HandoverRow, AuditRow};

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
        // Office (C.Office projections)
        .route("/office/entities", get(list_entities))
        .route("/office/entities/:entity_id", get(get_entity))
        .route("/office/entities/:entity_id/sessions", get(get_entity_sessions))
        .route("/office/entities/:entity_id/handovers", get(get_entity_handovers))
        .route("/office/entities/:entity_id/handovers/latest", get(get_latest_handover))
        .route("/office/audit", get(list_audit))
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

// =============================================================================
// OFFICE PROJECTION ROUTES
// =============================================================================

/// Query params for Office audit
#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub entity_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: Option<String>,
    pub limit: Option<i64>,
}

/// GET /query/office/entities — List all LLM entities
async fn list_entities(
    State(state): State<ProjectionState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<EntityRow>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(100);

    let entities: Vec<EntityRow> = sqlx::query_as!(
        EntityRow,
        r#"
        SELECT entity_id, name, entity_type, public_key, status,
               constitution, baseline_narrative, 
               total_sessions, total_tokens_used,
               created_at_ms, updated_at_ms
        FROM office_entities
        ORDER BY created_at_ms DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: entities }))
}

/// GET /query/office/entities/:entity_id — Get single entity
async fn get_entity(
    State(state): State<ProjectionState>,
    Path(entity_id): Path<String>,
) -> Result<Json<ApiResponse<EntityRow>>, (StatusCode, String)> {
    let entity: EntityRow = sqlx::query_as!(
        EntityRow,
        r#"
        SELECT entity_id, name, entity_type, public_key, status,
               constitution, baseline_narrative, 
               total_sessions, total_tokens_used,
               created_at_ms, updated_at_ms
        FROM office_entities
        WHERE entity_id = $1
        "#,
        entity_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Entity not found".to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: entity }))
}

/// GET /query/office/entities/:entity_id/sessions — Entity session history
async fn get_entity_sessions(
    State(state): State<ProjectionState>,
    Path(entity_id): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<SessionRow>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(100);

    let sessions: Vec<SessionRow> = sqlx::query_as!(
        SessionRow,
        r#"
        SELECT session_id, entity_id, session_type, mode, token_budget,
               tokens_used, duration_ms, status, started_at_ms, completed_at_ms
        FROM office_sessions
        WHERE entity_id = $1
        ORDER BY started_at_ms DESC
        LIMIT $2
        "#,
        entity_id,
        limit
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: sessions }))
}

/// GET /query/office/entities/:entity_id/handovers — Handover history
async fn get_entity_handovers(
    State(state): State<ProjectionState>,
    Path(entity_id): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<HandoverRow>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(50);

    let handovers: Vec<HandoverRow> = sqlx::query_as!(
        HandoverRow,
        r#"
        SELECT handover_id, entity_id, session_id, content, created_at_ms
        FROM office_handovers
        WHERE entity_id = $1
        ORDER BY created_at_ms DESC
        LIMIT $2
        "#,
        entity_id,
        limit
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: handovers }))
}

/// GET /query/office/entities/:entity_id/handovers/latest — Latest handover
async fn get_latest_handover(
    State(state): State<ProjectionState>,
    Path(entity_id): Path<String>,
) -> Result<Json<ApiResponse<Option<HandoverRow>>>, (StatusCode, String)> {
    let handover: Option<HandoverRow> = sqlx::query_as!(
        HandoverRow,
        r#"
        SELECT handover_id, entity_id, session_id, content, created_at_ms
        FROM office_handovers
        WHERE entity_id = $1
        ORDER BY created_at_ms DESC
        LIMIT 1
        "#,
        entity_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: handover }))
}

/// GET /query/office/audit — Audit trail
async fn list_audit(
    State(state): State<ProjectionState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<ApiResponse<Vec<AuditRow>>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(100).min(500);

    // Build dynamic query based on filters
    let mut sql = String::from(
        "SELECT audit_id, entity_id, session_id, job_id, trace_id, 
                event_type, event_data, created_at_ms
         FROM office_audit_log WHERE 1=1"
    );
    
    if query.entity_id.is_some() {
        sql.push_str(" AND entity_id = $1");
    }
    if query.session_id.is_some() {
        sql.push_str(" AND session_id = $2");
    }
    if query.event_type.is_some() {
        sql.push_str(" AND event_type = $3");
    }
    
    sql.push_str(" ORDER BY created_at_ms DESC LIMIT $4");

    // For simplicity, use a simpler query
    let audits: Vec<AuditRow> = sqlx::query_as!(
        AuditRow,
        r#"
        SELECT audit_id, entity_id, session_id, job_id, trace_id,
               event_type, event_data, created_at_ms
        FROM office_audit_log
        ORDER BY created_at_ms DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ApiResponse { ok: true, data: audits }))
}

