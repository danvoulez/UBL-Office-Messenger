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
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;
use tracing::{error, info};
use uuid::Uuid;

use crate::db::PgLedger;
use crate::messenger_gateway::{idempotency::IdempotencyStore, office_client::OfficeClient, sse::GatewaySSE};

use super::projections::GatewayProjections;

// Reuse helper from messenger_v1
use crate::messenger_v1::{get_user_from_session, UserInfo};

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
    let idempotency = Arc::new(IdempotencyStore::new());
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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
    let tenant_id = "default"; // TODO: Extract from session
    let idempotency_key = req.idempotency_key.clone().unwrap_or_else(|| {
        idempotency::IdempotencyStore::generate_key(
            tenant_id,
            "post_message",
            &conversation_id,
            &Uuid::new_v4().to_string(),
        )
    });
    
    if let Some(record) = state.idempotency.check(&idempotency_key) {
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
    let atom = serde_json::json!({
        "content_hash": content_hash,
        "conversation_id": conversation_id.clone(),
        "created_at": now_iso,
        "from": user.sid,
        "id": message_id.clone(),
        "message_type": req.message_type.as_deref().unwrap_or("text"),
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
            link_hash: "0x00".to_string(),
            ts_unix_ms: 0,
        });
    
    // Build link draft
    let link = crate::db::LinkDraft {
        version: 1,
        container_id: container_id.to_string(),
        expected_sequence: container_state.sequence + 1,
        previous_hash: container_state.entry_hash.clone(),
        atom_hash: atom_hash.clone(),
        atom: Some(atom.clone()),
        intent_class: "Observation".to_string(),
        physics_delta: "0".to_string(),
        author_pubkey: user.sid.clone(),
        signature: "placeholder".to_string(), // TODO: Use actual signature
        pact: None,
    };
    
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
            
            let record = idempotency::IdempotencyRecord {
                status: "completed".to_string(),
                response_body: Some(serde_json::to_value(&response).unwrap()),
                created_event_ids: office_resp.event_ids.clone(),
                created_at: OffsetDateTime::now_utc(),
            };
            state.idempotency.store(idempotency_key, record);
            
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
    let _user = get_user_from_session(&state.pool, &headers).await
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;
    
    // 2. Check idempotency
    let tenant_id = "default"; // TODO: Extract from session
    let idempotency_key = req.idempotency_key.clone().unwrap_or_else(|| {
        idempotency::IdempotencyStore::generate_key(
            tenant_id,
            "job_action",
            &job_id,
            &Uuid::new_v4().to_string(),
        )
    });
    
    if let Some(record) = state.idempotency.check(&idempotency_key) {
        if record.status == "completed" {
            if let Some(response) = record.response_body {
                return Ok(Json(serde_json::from_value(response)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?));
            }
        }
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
            let record = idempotency::IdempotencyRecord {
                status: "completed".to_string(),
                response_body: Some(serde_json::to_value(JobActionResponse {
                    success: office_resp.success,
                    event_ids: office_resp.event_ids.clone(),
                }).unwrap()),
                created_event_ids: office_resp.event_ids.clone(),
                created_at: time::OffsetDateTime::now_utc(),
            };
            state.idempotency.store(idempotency_key, record);
            
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
    
    // Query job header from projection_jobs
    let job_row = sqlx::query!(
        r#"
        SELECT job_id, title, goal, state, owner_entity_id, available_actions
        FROM projection_jobs
        WHERE tenant_id = $1 AND job_id = $2
        "#,
        tenant_id,
        job_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Job not found".to_string()))?;
    
    // Query timeline events
    let job_events = crate::projections::JobEventsProjection::new(state.pool.clone());
    let timeline = job_events.get_timeline(&tenant_id, &job_id, 100).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Query artifacts
    let artifacts_proj = crate::projections::ArtifactsProjection::new(state.pool.clone());
    let artifacts = artifacts_proj.get_artifacts(&tenant_id, &job_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Get owner entity info
    let owner_entity = sqlx::query!(
        r#"
        SELECT id, display_name, kind
        FROM id_subjects
        WHERE sid = $1
        "#,
        job_row.owner_entity_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(|r| serde_json::json!({
        "entity_id": r.id,
        "display_name": r.display_name,
        "kind": r.kind,
    }))
    .unwrap_or_else(|| serde_json::json!({
        "entity_id": job_row.owner_entity_id,
        "display_name": "Unknown",
        "kind": "unknown",
    }));
    
    // Parse available_actions
    let available_actions: Vec<serde_json::Value> = job_row.available_actions
        .as_array()
        .cloned()
        .unwrap_or_default();
    
    Ok(Json(JobResponse {
        job_id: job_row.job_id,
        title: job_row.title,
        goal: job_row.goal,
        state: job_row.state,
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

