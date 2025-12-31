//! Gateway Routes
//!
//! REST API endpoints for Messenger Gateway.
//! Handles command routing, idempotency, and projection queries.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use time::OffsetDateTime;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::db::PgLedger;
use crate::messenger_gateway::{idempotency::IdempotencyStore, office_client::OfficeClient, sse::GatewaySSE};

use super::projections::GatewayProjections;

// Reuse helpers from messenger_v1
use crate::messenger_v1::{get_user_from_session, UserInfo};

// Fix #1: Import signing function for real Ed25519 signatures
use crate::messenger_v1::sign_link_draft;

// ============================================================================
// STATE
// ============================================================================

#[derive(Clone)]
pub struct GatewayState {
    pub pool: PgPool,
    pub ledger: PgLedger,
    pub office_client: Arc<OfficeClient>,
    pub idempotency: Arc<IdempotencyStore>,
    pub projections: Arc<GatewayProjections>,
}

// ============================================================================
// ROUTES
// ============================================================================

pub fn routes(pool: PgPool, office_url: String) -> Router {
    let ledger = PgLedger::new(pool.clone());
    let office_client = Arc::new(OfficeClient::new(office_url));
    // Fix #4: Persistent idempotency backed by Postgres
    let idempotency = Arc::new(IdempotencyStore::new(pool.clone()));
    let projections = Arc::new(GatewayProjections::new(pool.clone()));
    
    let state = GatewayState {
        pool,
        ledger,
        office_client,
        idempotency,
        projections,
    };
    
    Router::new()
        // Commands
        .route("/v1/conversations/:id/messages", post(post_message))
        .route("/v1/jobs/:id/actions", post(job_action))
        // Queries
        .route("/v1/conversations/:id/timeline", get(get_timeline))
        .route("/v1/jobs/:id", get(get_job))
        // SSE
        .route("/v1/stream", get(get_stream))
        .with_state(state)
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
struct PostMessageRequest {
    content: String,
    message_type: Option<String>,
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostMessageResponse {
    message_id: String,
    hash: String,
    sequence: i64,
    action: String, // "committed" | "office_processing"
}

#[derive(Debug, Deserialize)]
struct JobActionRequest {
    action_type: String,
    button_id: String,
    card_id: String,
    input_data: Option<serde_json::Value>,
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobActionResponse {
    success: bool,
    event_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TimelineResponse {
    items: Vec<serde_json::Value>,
    cursor: String,
}

#[derive(Debug, Serialize)]
struct JobResponse {
    job_id: String,
    title: String,
    goal: String,
    state: String,
    owner: serde_json::Value,
    available_actions: Vec<serde_json::Value>,
    timeline: Vec<serde_json::Value>,
    artifacts: Vec<serde_json::Value>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// POST /v1/conversations/:id/messages
/// Send a message via Gateway → Office → UBL
async fn post_message(
    State(state): State<GatewayState>,
    Path(conversation_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<PostMessageRequest>,
) -> Result<Json<PostMessageResponse>, (StatusCode, String)> {
    // 1. Get user from session
    let user = get_user_from_session(&state.pool, &headers).await
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;
    
    // 2. Check idempotency
    // Zona Schengen: Use tenant_id from session
    let tenant_id = user.tenant_id.as_deref().unwrap_or("default");
    let idempotency_key = req.idempotency_key.clone().unwrap_or_else(|| {
        crate::messenger_gateway::idempotency::IdempotencyStore::generate_key(
            tenant_id,
            "post_message",
            &conversation_id,
            &Uuid::new_v4().to_string(),
        )
    });
    
    // Fix #4: Async idempotency check (Postgres-backed)
    if let Some(record) = state.idempotency.check(&idempotency_key).await {
        if record.status == "completed" {
            // Return cached response
            if let Some(response) = record.response_body {
                return Ok(Json(serde_json::from_value(response)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?));
            }
        }
    }
    
    // 3. Generate message ID
    let message_id = format!("msg_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
    
    // 4. Commit message.created to UBL (C.Messenger) first
    let now = OffsetDateTime::now_utc();
    let now_iso = now.format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Time format error: {}", e)))?;
    
    // Hash content for privacy
    let content_hash = crate::messenger_v1::blake3_hex(&req.content);
    
    // Build canonical atom
    // Fix #5: Include tenant_id for proper isolation
    let atom = serde_json::json!({
        "content_hash": content_hash,
        "conversation_id": conversation_id.clone(),
        "created_at": now_iso,
        "from": user.sid,
        "id": message_id.clone(),
        "message_type": req.message_type.as_deref().unwrap_or("text"),
        "tenant_id": tenant_id,
        "type": "message.created"
    });
    
    // Canonicalize and hash
    let atom_bytes = ubl_atom::canonicalize(&atom)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Canonicalize error: {}", e)))?;
    let atom_hash = crate::messenger_v1::blake3_hex_bytes(&atom_bytes);
    
    // Get container state
    let container_id = "C.Messenger";
    let container_state = state.ledger.get_state(container_id).await
        .unwrap_or_else(|_| crate::db::LedgerEntry {
            container_id: container_id.to_string(),
            sequence: 0,
            entry_hash: "0x00".to_string(),
            previous_hash: "0x00".to_string(),
            link_hash: "0x00".to_string(),
            ts_unix_ms: 0,
        });
    
    // Build and SIGN link draft (Fix #1: Real Ed25519)
    let mut link = crate::db::LinkDraft {
        version: 1,
        container_id: container_id.to_string(),
        expected_sequence: container_state.sequence + 1,
        previous_hash: container_state.entry_hash.clone(),
        atom_hash: atom_hash.clone(),
        atom: Some(atom.clone()),
        intent_class: "Observation".to_string(),
        physics_delta: "0".to_string(),
        author_pubkey: String::new(), // Will be set by sign_link_draft
        signature: String::new(),     // Will be set by sign_link_draft
        pact: None,
    };
    sign_link_draft(&mut link);
    
    // Commit to ledger
    let entry = state.ledger.append(&link).await
        .map_err(|e| (StatusCode::CONFLICT, format!("Commit failed: {:?}", e)))?;
    
    // Store message content
    crate::messenger_v1::store_message_content(&state.pool, &message_id, &req.content, &content_hash).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 5. Call Office to ingest message
    let office_req = super::office_client::IngestMessageRequest {
        conversation_id: conversation_id.clone(),
        message_id: message_id.clone(),
        from: user.sid.clone(),
        content: req.content.clone(),
        tenant_id: tenant_id.to_string(),
    };
    
    match state.office_client.ingest_message(&office_req).await {
        Ok(office_resp) => {
            info!("✅ Office processed message: action={:?}", office_resp.action);
            
            // Store idempotency record
            let response = PostMessageResponse {
                message_id: message_id.clone(),
                hash: entry.entry_hash.clone(),
                sequence: entry.sequence,
                action: format!("{:?}", office_resp.action),
            };
            
            let record = crate::messenger_gateway::idempotency::IdempotencyRecord {
                status: "completed".to_string(),
                // Fix #15: Handle JSON serialization errors gracefully
                response_body: serde_json::to_value(&response).ok(),
                created_event_ids: office_resp.event_ids.clone(),
                created_at: OffsetDateTime::now_utc(),
            };
            // Fix #4: Async store (ignore errors - idempotency is best-effort)
            let _ = state.idempotency.store(idempotency_key, tenant_id, record).await;
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("❌ Office ingest_message failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// POST /v1/jobs/:id/actions
/// Handle job action via Gateway → Office
async fn job_action(
    State(state): State<GatewayState>,
    Path(job_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<JobActionRequest>,
) -> Result<Json<JobActionResponse>, (StatusCode, String)> {
    // 1. Get user from session
    let user = get_user_from_session(&state.pool, &headers).await
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;
    
    // 2. Check idempotency
    // Zona Schengen: Use tenant_id from session
    let tenant_id = user.tenant_id.as_deref().unwrap_or("default");
    let idempotency_key = req.idempotency_key.clone().unwrap_or_else(|| {
        crate::messenger_gateway::idempotency::IdempotencyStore::generate_key(
            tenant_id,
            "job_action",
            &job_id,
            &Uuid::new_v4().to_string(),
        )
    });
    
    // Fix #4: Async idempotency check (Postgres-backed)
    if let Some(record) = state.idempotency.check(&idempotency_key).await {
        if record.status == "completed" {
            if let Some(response) = record.response_body {
                return Ok(Json(serde_json::from_value(response)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?));
            }
        }
    }
    
    // Fix #6: Validate job exists and user has access via projection
    let job = state.projections.get_job(&job_id, tenant_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch job: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, format!("Job {} not found", job_id)))?;
    
    // Check user has access (is owner, participant, or has approval rights)
    let has_access = job.owner_entity_id.as_deref() == Some(&user.sid)
        || job.waiting_on.as_ref().map_or(false, |w| w.contains(&user.sid));
    
    if !has_access {
        warn!("🚫 User {} attempted action on job {} without access", user.sid, job_id);
        return Err((StatusCode::FORBIDDEN, "You don't have access to this job".to_string()));
    }
    
    // Fix #8: Validate FSM transition is legal before calling Office
    // This provides early validation at Gateway level
    let current_state = &job.state;
    let target_state = match req.action_type.as_str() {
        "approve" => Some("approved"),
        "reject" => Some("rejected"),
        "cancel" => Some("cancelled"),
        "complete" => Some("completed"),
        "start" => Some("in_progress"),
        "provide_input" => Some("in_progress"), // waiting_input → in_progress
        _ => None,
    };
    
    if let Some(to_state) = target_state {
        let policy_engine = crate::policy::PolicyEngine::new(state.pool.clone());
        if let Err(e) = policy_engine.validate_job_fsm(&job_id, current_state, to_state).await {
            warn!("🚫 FSM violation at Gateway: {} → {} (job: {})", current_state, to_state, job_id);
            return Err((StatusCode::BAD_REQUEST, format!("Invalid state transition: {}", e)));
        }
        info!("✅ FSM pre-check passed: {} → {} (job: {})", current_state, to_state, job_id);
    }
    
    // 3. Call Office to handle job action
    let office_req = super::office_client::JobActionRequest {
        job_id: job_id.clone(),
        action_type: req.action_type.clone(),
        button_id: req.button_id.clone(),
        card_id: req.card_id.clone(),
        input_data: req.input_data.clone(),
        tenant_id: tenant_id.to_string(),
    };
    
    match state.office_client.job_action(&office_req).await {
        Ok(office_resp) => {
            info!("✅ Office processed job action: success={}", office_resp.success);
            
            // Store idempotency record
            let record = crate::messenger_gateway::idempotency::IdempotencyRecord {
                status: "completed".to_string(),
                // Fix #15: Handle JSON serialization errors gracefully
                response_body: serde_json::to_value(JobActionResponse {
                    success: office_resp.success,
                    event_ids: office_resp.event_ids.clone(),
                }).ok(),
                created_event_ids: office_resp.event_ids.clone(),
                created_at: time::OffsetDateTime::now_utc(),
            };
            // Fix #4: Async store (ignore errors - idempotency is best-effort)
            let _ = state.idempotency.store(idempotency_key, tenant_id, record).await;
            
            Ok(Json(JobActionResponse {
                success: office_resp.success,
                event_ids: office_resp.event_ids,
            }))
        }
        Err(e) => {
            error!("❌ Office job_action failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// GET /v1/conversations/:id/timeline
/// Query timeline from projections
async fn get_timeline(
    State(state): State<GatewayState>,
    Path(conversation_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<TimelineResponse>, (StatusCode, String)> {
    // TODO: Query projection_timeline_items
    // For now, return empty
    Ok(Json(TimelineResponse {
        items: vec![],
        cursor: "0:0".to_string(),
    }))
}

/// GET /v1/jobs/:id
/// Get job details for drawer
async fn get_job(
    State(state): State<GatewayState>,
    Path(job_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    let tenant_id = params.get("tenant_id").cloned().unwrap_or_else(|| "default".to_string());
    
    // Query job header from projection_jobs (using dynamic query)
    let job_row = sqlx::query(
        r#"
        SELECT job_id, title, goal, state, owner_entity_id, available_actions
        FROM projection_jobs
        WHERE tenant_id = $1 AND job_id = $2
        "#
    )
    .bind(&tenant_id)
    .bind(&job_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Job not found".to_string()))?;
    
    // Extract fields from row
    let job_job_id: String = job_row.get("job_id");
    let job_title: String = job_row.get("title");
    let job_goal: String = job_row.get("goal");
    let job_state: String = job_row.get("state");
    let owner_entity_id: String = job_row.get("owner_entity_id");
    let available_actions_json: Option<serde_json::Value> = job_row.try_get("available_actions").ok();
    
    // Query timeline events
    let job_events = crate::projections::JobEventsProjection::new(state.pool.clone());
    let timeline = job_events.get_timeline(&tenant_id, &job_id, 100).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Query artifacts
    let artifacts_proj = crate::projections::ArtifactsProjection::new(state.pool.clone());
    let artifacts = artifacts_proj.get_artifacts(&tenant_id, &job_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Get owner entity info (using dynamic query)
    let owner_entity = sqlx::query(
        r#"
        SELECT sid, display_name, kind
        FROM id_subject
        WHERE sid = $1
        "#
    )
    .bind(&owner_entity_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(|r| {
        let sid: String = r.get("sid");
        let display_name: String = r.get("display_name");
        let kind: String = r.get("kind");
        serde_json::json!({
            "entity_id": sid,
            "display_name": display_name,
            "kind": kind,
        })
    })
    .unwrap_or_else(|| serde_json::json!({
        "entity_id": owner_entity_id,
        "display_name": "Unknown",
        "kind": "unknown",
    }));
    
    // Parse available_actions
    let available_actions: Vec<serde_json::Value> = available_actions_json
        .as_ref()
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    
    Ok(Json(JobResponse {
        job_id: job_job_id,
        title: job_title,
        goal: job_goal,
        state: job_state,
        owner: owner_entity,
        available_actions,
        timeline,
        artifacts,
    }))
}

/// GET /v1/stream
/// SSE delta stream
async fn get_stream(
    State(state): State<GatewayState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let tenant_id = params.get("tenant_id").cloned().unwrap_or_else(|| "default".to_string());
    let cursor = params.get("cursor").cloned();
    
    let (gateway_sse, sse) = super::sse::GatewaySSE::new();
    
    // Emit hello event with current cursor
    let current_cursor = cursor.unwrap_or_else(|| "0:0".to_string());
    let _ = gateway_sse.emit(super::sse::DeltaEvent::Hello {
        cursor: current_cursor.clone(),
    }).await;
    
    // TODO: Subscribe to UBL SSE tail and forward deltas
    // For now, emit heartbeat periodically
    let sse_clone = gateway_sse.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            let _ = sse_clone.emit(super::sse::DeltaEvent::Heartbeat).await;
        }
    });
    
    sse
}

