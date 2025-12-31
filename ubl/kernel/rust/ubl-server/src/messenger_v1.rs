//! C.Messenger Boundary API v1
//!
//! Translates frontend semantic requests to ubl-link commits.
//! This is the TDLN layer for the Messenger container.
//!
//! Endpoints:
//! - GET  /messenger/bootstrap      → Initial state aggregation
//! - POST /messenger/messages       → Send message (commit to ledger)
//! - POST /messenger/conversations  → Create workstream
//! - POST /messenger/jobs/:id/approve → Approve job (commit to C.Jobs)
//! - POST /messenger/jobs/:id/reject  → Reject job (commit to C.Jobs)
//!
//! All mutations follow the pattern:
//! 1. Validate request
//! 2. Build canonical ubl-atom
//! 3. Create ubl-link (signed with REAL Ed25519)
//! 4. POST /link/commit internally
//! 5. Return result
//!
//! ## Signature Strategy (Fix #1)
//! 
//! The boundary layer uses a persistent "boundary" key to sign commits.
//! This key is stored in the keystore and persists across restarts.
//! 
//! In production, consider:
//! - Frontend signing via WebAuthn PRF (user's passkey)
//! - Delegated signing (Office signs on behalf of user with consent)

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth;
use crate::db::{LinkDraft, PgLedger};
use crate::keystore;
use crate::projections::{JobsProjection, MessagesProjection};

// ============================================================================
// STATE
// ============================================================================

#[derive(Clone)]
pub struct MessengerState {
    pub pool: PgPool,
    pub ledger: PgLedger,
}

// ============================================================================
// ROUTES
// ============================================================================

pub fn routes(pool: PgPool) -> Router {
    let ledger = PgLedger::new(pool.clone());
    let state = MessengerState { pool, ledger };
    
    Router::new()
        // Aggregation
        .route("/messenger/bootstrap", get(bootstrap))
        // Mutations
        .route("/messenger/messages", post(send_message))
        .route("/messenger/conversations", get(list_conversations))
        .route("/messenger/conversations", post(create_conversation))
        .route("/messenger/jobs/:job_id/approve", post(approve_job))
        .route("/messenger/jobs/:job_id/reject", post(reject_job))
        // Entities
        .route("/messenger/entities", get(list_entities))
        .with_state(state)
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize)]
pub struct BootstrapResponse {
    pub user: Option<UserInfo>,
    pub entities: Vec<EntityInfo>,
    pub conversations: Vec<ConversationInfo>,
    pub messages: Vec<MessageInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserInfo {
    pub sid: String,
    pub display_name: String,
    pub kind: String,
    pub tenant_id: Option<String>,  // Zona Schengen: tenant from session
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EntityInfo {
    pub id: String,
    pub display_name: String,
    pub kind: String,
    pub avatar_url: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ConversationInfo {
    pub id: String,
    pub name: Option<String>,
    pub is_group: bool,
    pub participants: Vec<String>,
    pub last_message: Option<String>,
    pub last_message_at: Option<OffsetDateTime>,
    pub unread_count: i64,
}

#[derive(Debug, Serialize)]
pub struct MessageInfo {
    pub id: String,
    pub conversation_id: String,
    pub from_id: String,
    pub content: String,
    pub content_hash: String,
    pub message_type: String,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: String,
    pub content: String,
    #[serde(default = "default_message_type")]
    pub message_type: String,
}

fn default_message_type() -> String { "text".to_string() }

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    pub hash: String,
    pub sequence: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub name: Option<String>,
    pub participants: Vec<String>,
    #[serde(default)]
    pub is_group: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateConversationResponse {
    pub id: String,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalDecisionRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApprovalDecisionResponse {
    pub job_id: String,
    pub decision: String,
    pub hash: String,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// GET /messenger/bootstrap
/// Aggregates initial state for the frontend
async fn bootstrap(
    State(state): State<MessengerState>,
    headers: HeaderMap,
) -> Result<Json<BootstrapResponse>, (StatusCode, String)> {
    // 1. Get current user from session
    let user = get_user_from_session(&state.pool, &headers).await;
    
    // 2. Get entities
    let entities = get_entities(&state.pool).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 3. Get conversations for user
    let user_id = user.as_ref().map(|u| u.sid.as_str()).unwrap_or("demo");
    let conversations = get_user_conversations(&state.pool, user_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 4. Get recent messages (aggregate from all user's conversations)
    let mut messages = Vec::new();
    for conv in &conversations {
        let conv_messages = get_conversation_messages(&state.pool, &conv.id, 50).await
            .unwrap_or_default();
        messages.extend(conv_messages);
    }
    
    Ok(Json(BootstrapResponse {
        user,
        entities,
        conversations,
        messages,
    }))
}

/// POST /messenger/messages
/// Send a message (commits message.created to C.Messenger)
async fn send_message(
    State(state): State<MessengerState>,
    headers: HeaderMap,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, (StatusCode, String)> {
    // 1. Get sender from session
    let user = get_user_from_session(&state.pool, &headers).await
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;
    
    // 2. Generate message ID
    let message_id = format!("msg_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
    let now = OffsetDateTime::now_utc();
    let now_iso = now.format(&time::format_description::well_known::Rfc3339).unwrap();
    
    // 3. Hash the content (for privacy - ledger stores hash, not content)
    let content_hash = blake3_hex(&req.content);
    
    // 4. Build canonical atom (SPEC-UBL-ATOM v1.0 compliant)
    // Keys MUST be sorted lexicographically
    let atom = serde_json::json!({
        "content_hash": content_hash,
        "conversation_id": req.conversation_id,
        "created_at": now_iso,
        "from": user.sid,
        "id": message_id,
        "message_type": req.message_type,
        "type": "message.created"
    });
    
    // 5. Canonicalize and hash
    let atom_bytes = ubl_atom::canonicalize(&atom)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Canonicalize error: {}", e)))?;
    let atom_hash = blake3_hex_bytes(&atom_bytes);
    
    // 6. Get container state for sequence
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
    
    // 7. Build and SIGN the link (Fix #1: Real Ed25519)
    let mut link = LinkDraft {
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
    
    // 8. Commit to ledger
    let entry = state.ledger.append(&link).await
        .map_err(|e| (StatusCode::CONFLICT, format!("Commit failed: {:?}", e)))?;
    
    // 9. Also store the actual content in ledger_atom for retrieval
    store_message_content(&state.pool, &message_id, &req.content, &content_hash).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(SendMessageResponse {
        message_id,
        hash: entry.entry_hash,
        sequence: entry.sequence,
    }))
}

/// GET /messenger/conversations
async fn list_conversations(
    State(state): State<MessengerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConversationInfo>>, (StatusCode, String)> {
    let user = get_user_from_session(&state.pool, &headers).await;
    let user_id = user.as_ref().map(|u| u.sid.as_str()).unwrap_or("demo");
    
    let conversations = get_user_conversations(&state.pool, user_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(conversations))
}

/// POST /messenger/conversations
/// Create a new conversation/workstream
async fn create_conversation(
    State(state): State<MessengerState>,
    headers: HeaderMap,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<CreateConversationResponse>, (StatusCode, String)> {
    // 1. Get creator from session
    let user = get_user_from_session(&state.pool, &headers).await
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;
    
    // 2. Generate conversation ID
    let conv_id = format!("conv_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
    let now = OffsetDateTime::now_utc();
    let now_iso = now.format(&time::format_description::well_known::Rfc3339).unwrap();
    
    // 3. Build canonical atom
    let mut participants = req.participants.clone();
    if !participants.contains(&user.sid) {
        participants.push(user.sid.clone());
    }
    participants.sort(); // Canonical order
    
    let atom = serde_json::json!({
        "created_at": now_iso,
        "created_by": user.sid,
        "id": conv_id,
        "is_group": req.is_group || participants.len() > 2,
        "name": req.name.clone().unwrap_or_default(),
        "participants": participants,
        "type": "conversation.created"
    });
    
    // 4. Canonicalize and hash
    let atom_bytes = ubl_atom::canonicalize(&atom)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Canonicalize error: {}", e)))?;
    let atom_hash = blake3_hex_bytes(&atom_bytes);
    
    // 5. Get container state
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
    
    // 6. Build and SIGN the link (Fix #1: Real Ed25519)
    let mut link = LinkDraft {
        version: 1,
        container_id: container_id.to_string(),
        expected_sequence: container_state.sequence + 1,
        previous_hash: container_state.entry_hash.clone(),
        atom_hash: atom_hash.clone(),
        atom: Some(atom),
        intent_class: "Observation".to_string(),
        physics_delta: "0".to_string(),
        author_pubkey: String::new(), // Will be set by sign_link_draft
        signature: String::new(),     // Will be set by sign_link_draft
        pact: None,
    };
    sign_link_draft(&mut link);
    
    let entry = state.ledger.append(&link).await
        .map_err(|e| (StatusCode::CONFLICT, format!("Commit failed: {:?}", e)))?;
    
    Ok(Json(CreateConversationResponse {
        id: conv_id,
        hash: entry.entry_hash,
    }))
}

/// POST /messenger/jobs/:job_id/approve
async fn approve_job(
    State(state): State<MessengerState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(req): Json<ApprovalDecisionRequest>,
) -> Result<Json<ApprovalDecisionResponse>, (StatusCode, String)> {
    job_decision(state, headers, job_id, "approved", req.reason).await
}

/// POST /messenger/jobs/:job_id/reject
async fn reject_job(
    State(state): State<MessengerState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(req): Json<ApprovalDecisionRequest>,
) -> Result<Json<ApprovalDecisionResponse>, (StatusCode, String)> {
    job_decision(state, headers, job_id, "rejected", req.reason).await
}

/// Common logic for approve/reject
async fn job_decision(
    state: MessengerState,
    headers: HeaderMap,
    job_id: String,
    decision: &str,
    reason: Option<String>,
) -> Result<Json<ApprovalDecisionResponse>, (StatusCode, String)> {
    // 1. Get user
    let user = get_user_from_session(&state.pool, &headers).await
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;
    
    // 2. Get pending approval for this job
    let jobs_projection = JobsProjection::new(state.pool.clone());
    let approvals = jobs_projection.get_pending_approvals(&job_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let approval = approvals.first()
        .ok_or((StatusCode::NOT_FOUND, "No pending approval for this job".to_string()))?;
    
    // 3. Build approval.decided atom
    let now = OffsetDateTime::now_utc();
    let now_iso = now.format(&time::format_description::well_known::Rfc3339).unwrap();
    
    let atom = serde_json::json!({
        "approval_id": approval.approval_id,
        "decided_at": now_iso,
        "decided_by": user.sid,
        "decision": decision,
        "job_id": job_id,
        "reason": reason.unwrap_or_default(),
        "type": "approval.decided"
    });
    
    // 4. Canonicalize
    let atom_bytes = ubl_atom::canonicalize(&atom)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Canonicalize error: {}", e)))?;
    let atom_hash = blake3_hex_bytes(&atom_bytes);
    
    // 5. Commit to C.Jobs
    let container_id = "C.Jobs";
    let container_state = state.ledger.get_state(container_id).await
        .unwrap_or_else(|_| crate::db::LedgerEntry {
            container_id: container_id.to_string(),
            sequence: 0,
            entry_hash: "0x00".to_string(),
            previous_hash: "0x00".to_string(),
            link_hash: "0x00".to_string(),
            ts_unix_ms: 0,
        });
    
    // 6. Build and SIGN the link (Fix #1: Real Ed25519)
    let mut link = LinkDraft {
        version: 1,
        container_id: container_id.to_string(),
        expected_sequence: container_state.sequence + 1,
        previous_hash: container_state.entry_hash.clone(),
        atom_hash: atom_hash.clone(),
        atom: Some(atom),
        intent_class: "Observation".to_string(),
        physics_delta: "0".to_string(),
        author_pubkey: String::new(), // Will be set by sign_link_draft
        signature: String::new(),     // Will be set by sign_link_draft
        pact: None,
    };
    sign_link_draft(&mut link);
    
    let entry = state.ledger.append(&link).await
        .map_err(|e| (StatusCode::CONFLICT, format!("Commit failed: {:?}", e)))?;
    
    Ok(Json(ApprovalDecisionResponse {
        job_id,
        decision: decision.to_string(),
        hash: entry.entry_hash,
    }))
}

/// GET /messenger/entities
async fn list_entities(
    State(state): State<MessengerState>,
) -> Result<Json<Vec<EntityInfo>>, (StatusCode, String)> {
    let entities = get_entities(&state.pool).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(entities))
}

// ============================================================================
// HELPERS
// ============================================================================

pub async fn get_user_from_session(pool: &PgPool, headers: &HeaderMap) -> Option<UserInfo> {
    // Extract token from Authorization header or cookie
    let token = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            headers.get("cookie")
                .and_then(|h| h.to_str().ok())
                .and_then(|c| {
                    c.split(';')
                        .find_map(|part| {
                            let mut kv = part.trim().splitn(2, '=');
                            match (kv.next(), kv.next()) {
                                (Some("session"), Some(v)) => Some(v.to_string()),
                                _ => None
                            }
                        })
                })
        });
    
    let token = token?;
    
    // Validate session
    let session = crate::auth::session_db::get_valid(pool, &token).await.ok()??;
    let sid_str = session.sid.to_string();
    let tenant_id = session.tenant_id.clone();  // Extract tenant from session (Zona Schengen)
    
    // Get subject details
    let subject = crate::id_db::get_subject(pool, &sid_str).await.ok()??;
    
    Some(UserInfo {
        sid: subject.sid,
        display_name: subject.display_name,
        kind: subject.kind,
        tenant_id,
    })
}

async fn get_entities(pool: &PgPool) -> Result<Vec<EntityInfo>, sqlx::Error> {
    // Get all registered subjects
    let entities: Vec<EntityInfo> = sqlx::query_as(
        r#"
        SELECT sid as id, display_name, kind, NULL as avatar_url, 'online' as status
        FROM id_subjects
        WHERE kind IN ('person', 'llm', 'app')
        ORDER BY display_name
        "#
    )
    .fetch_all(pool)
    .await?;
    
    Ok(entities)
}

async fn get_user_conversations(pool: &PgPool, user_id: &str) -> Result<Vec<ConversationInfo>, sqlx::Error> {
    // Query projection_conversations (if exists) or derive from messages
    let conversations: Vec<ConversationInfo> = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (conversation_id)
            conversation_id as id,
            NULL as name,
            false as is_group,
            ARRAY[from_id] as participants,
            NULL as last_message,
            timestamp as last_message_at,
            0::bigint as unread_count
        FROM projection_messages
        ORDER BY conversation_id, timestamp DESC
        LIMIT 50
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    Ok(conversations)
}

async fn get_conversation_messages(pool: &PgPool, conversation_id: &str, limit: i64) -> Result<Vec<MessageInfo>, sqlx::Error> {
    let projection = MessagesProjection::new(pool.clone());
    let messages = projection.get_messages_by_conversation(conversation_id, limit, None).await?;
    
    // Convert and fetch content
    let mut result = Vec::new();
    for msg in messages {
        let content = get_message_content(pool, &msg.message_id).await
            .unwrap_or_else(|_| format!("[Content: {}]", &msg.content_hash[..8]));
        
        result.push(MessageInfo {
            id: msg.message_id,
            conversation_id: msg.conversation_id,
            from_id: msg.from_id,
            content,
            content_hash: msg.content_hash,
            message_type: msg.message_type,
            timestamp: msg.timestamp,
        });
    }
    
    Ok(result)
}

pub async fn store_message_content(pool: &PgPool, message_id: &str, content: &str, content_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO message_content (message_id, content, content_hash, created_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (message_id) DO NOTHING
        "#
    )
    .bind(message_id)
    .bind(content)
    .bind(content_hash)
    .execute(pool)
    .await?;
    
    Ok(())
}

async fn get_message_content(pool: &PgPool, message_id: &str) -> Result<String, sqlx::Error> {
    let content: Option<String> = sqlx::query_scalar(
        "SELECT content FROM message_content WHERE message_id = $1"
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(content.unwrap_or_default())
}

pub fn blake3_hex(data: &str) -> String {
    let hash = blake3::hash(data.as_bytes());
    hash.to_hex().to_string()
}

pub fn blake3_hex_bytes(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

// ============================================================================
// SIGNING (Fix #1: Real Ed25519 Signatures)
// ============================================================================

/// Key ID for the boundary signing key
const BOUNDARY_KEY_ID: &str = "boundary";

/// Sign a link draft and return the completed link
/// 
/// Uses the persistent "boundary" key from keystore.
/// This ensures signatures are real Ed25519 and commits pass membrane validation.
pub fn sign_link_draft(link: &mut LinkDraft) {
    // Build canonical signing data (must match main.rs verification)
    let signing_data = serde_json::json!({
        "version": link.version,
        "container_id": link.container_id,
        "expected_sequence": link.expected_sequence,
        "previous_hash": link.previous_hash,
        "atom_hash": link.atom_hash,
        "intent_class": link.intent_class,
        "physics_delta": link.physics_delta,
        "pact": link.pact,
    });
    
    // Canonicalize (sorted keys, no whitespace)
    let signing_bytes = ubl_atom::canonicalize(&signing_data)
        .expect("Failed to canonicalize link for signing");
    
    // Get public key for author field
    link.author_pubkey = keystore::get_public_key_hex(BOUNDARY_KEY_ID);
    
    // Sign with boundary key
    // Note: keystore::sign returns "ed25519:base64" format, but main.rs expects hex
    // So we use the underlying key directly
    let key = keystore::load_or_create(BOUNDARY_KEY_ID);
    link.signature = ubl_kernel::sign(&key, &signing_bytes);
}

/// Get the public key of the boundary signer
/// Useful for registering in id_subjects as an authorized signer
pub fn get_boundary_pubkey() -> String {
    keystore::get_public_key_hex(BOUNDARY_KEY_ID)
}

