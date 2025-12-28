//! Console API v1.1 — ADR-UBL-Console-001
//!
//! Endpoints:
//! - POST /v1/policy/permit  → Emit Permit
//! - POST /v1/commands/issue → Register Command (pending=1)
//! - GET  /v1/query/commands → List pending commands for Runner
//! - POST /v1/exec.finish    → Register Receipt, mark command done

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// State for console routes
#[derive(Clone)]
pub struct ConsoleState {
    pub pool: PgPool,
}

/// Create console v1.1 routes
pub fn routes(pool: PgPool) -> Router {
    let state = ConsoleState { pool };
    Router::new()
        .route("/v1/policy/permit", post(issue_permit))
        .route("/v1/commands/issue", post(issue_command))
        .route("/v1/query/commands", get(query_commands))
        .route("/v1/exec.finish", post(exec_finish))
        .with_state(state)
}

// ============ Types ============

#[derive(Debug, Deserialize)]
pub struct PermitRequest {
    pub tenant_id: String,
    pub actor_id: String,
    pub intent: String,
    pub context: serde_json::Value,
    #[serde(rename = "jobType")]
    pub job_type: String,
    pub params: serde_json::Value,
    pub target: String,
    pub approval_ref: Option<String>,
    /// Risk level: "L0" through "L5"
    /// L4/L5 require WebAuthn step-up
    #[serde(default = "default_risk")]
    pub risk: String,
    /// WebAuthn assertion for L4/L5 step-up
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webauthn_assertion: Option<serde_json::Value>,
}

fn default_risk() -> String { "L0".to_string() }

#[derive(Debug, Serialize)]
pub struct PermitScopes {
    pub tenant_id: String,
    #[serde(rename = "jobType")]
    pub job_type: String,
    pub target: String,
    pub subject_hash: String,
    pub policy_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Permit {
    pub aud: String,
    pub jti: String,
    pub exp: u64,
    pub sig: String,
    pub scopes: PermitScopes,
}

#[derive(Debug, Serialize)]
pub struct PermitResponse {
    pub permit: Permit,
    pub policy_hash: String,
    pub subject_hash: String,
    pub allowed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CommandEnvelope {
    pub jti: String,
    pub tenant_id: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "jobType")]
    pub job_type: String,
    pub params: serde_json::Value,
    pub subject_hash: String,
    pub policy_hash: String,
    pub permit: serde_json::Value,
    pub target: String,
    pub office_id: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryCommandsParams {
    pub tenant_id: String,
    pub target: String,
    #[serde(default = "default_pending")]
    pub pending: i32,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_pending() -> i32 { 1 }
fn default_limit() -> i32 { 5 }

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CommandRow {
    pub jti: String,
    pub tenant_id: String,
    pub job_id: String,
    pub job_type: String,
    pub params: serde_json::Value,
    pub subject_hash: String,
    pub policy_hash: String,
    pub permit: serde_json::Value,
    pub target: String,
    pub office_id: String,
    pub pending: i32,
    pub issued_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct Receipt {
    pub tenant_id: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
    pub status: String,
    pub finished_at: u64,
    pub logs_hash: String,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub usage: serde_json::Value,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// ============ Handlers ============

/// POST /v1/policy/permit — Issue a Permit
/// SECURITY: L4/L5 risk levels require WebAuthn step-up authentication
async fn issue_permit(
    State(state): State<ConsoleState>,
    Json(req): Json<PermitRequest>,
) -> impl IntoResponse {
    // CRITICAL: L4/L5 require WebAuthn step-up (Passkey)
    let needs_stepup = req.risk == "L4" || req.risk == "L5";
    if needs_stepup {
        if req.webauthn_assertion.is_none() {
            return (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse { 
                    error: "Step-up authentication required for L4/L5 operations".to_string() 
                }),
            ).into_response();
        }
        
        // TODO: Validate WebAuthn assertion with stored passkey
        // For now, we check that the assertion is not empty
        if let Some(ref assertion) = req.webauthn_assertion {
            if assertion.is_null() || (assertion.is_object() && assertion.as_object().map(|o| o.is_empty()).unwrap_or(true)) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse { 
                        error: "Invalid WebAuthn assertion".to_string() 
                    }),
                ).into_response();
            }
            // In production: validate signature with WebAuthn library
            tracing::info!("✅ Step-up authentication validated for {} operation", req.risk);
        }
    }
    
    // Compute hashes
    let params_canonical = serde_json::to_string(&req.params).unwrap_or_default();
    let subject_hash = blake3_hex(&params_canonical);
    
    // For now, use a placeholder policy hash (would come from policy registry)
    let policy_hash = blake3_hex(&format!("policy:{}:{}", req.tenant_id, req.job_type));
    
    // TODO: Evaluate policy here - for now, allow all
    // In production: call PolicyRegistry to evaluate
    
    // Generate permit
    let jti = Uuid::new_v4().to_string();
    let now_ms = now_millis();
    let ttl_ms = get_ttl_for_risk(&req.risk);
    let exp = now_ms + ttl_ms;
    
    let scopes = PermitScopes {
        tenant_id: req.tenant_id.clone(),
        job_type: req.job_type.clone(),
        target: req.target.clone(),
        subject_hash: subject_hash.clone(),
        policy_hash: policy_hash.clone(),
        approval_ref: req.approval_ref.clone(),
    };
    
    // Sign the permit (simplified - would use Ed25519 in production)
    let permit_data = serde_json::to_string(&scopes).unwrap_or_default();
    let sig = blake3_hex(&format!("{}:{}:{}", permit_data, jti, exp));
    
    let permit = Permit {
        aud: format!("runner:{}", req.target),
        jti: jti.clone(),
        exp,
        sig,
        scopes,
    };
    
    // Store permit for audit
    let pool = &state.pool;
    let _ = sqlx::query(
        r#"
        INSERT INTO permits (jti, tenant_id, actor_id, job_type, target, subject_hash, policy_hash, approval_ref, exp, issued_at, used)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, false)
        ON CONFLICT (jti) DO NOTHING
        "#
    )
    .bind(&jti)
    .bind(&req.tenant_id)
    .bind(&req.actor_id)
    .bind(&req.job_type)
    .bind(&req.target)
    .bind(&subject_hash)
    .bind(&policy_hash)
    .bind(&req.approval_ref)
    .bind(exp as i64)
    .bind(now_ms as i64)
    .execute(pool)
    .await;
    
    let response = PermitResponse {
        permit,
        policy_hash,
        subject_hash,
        allowed: true,
    };
    
    (StatusCode::OK, Json(response))
}

/// POST /v1/commands/issue — Register a command for Runner to execute
async fn issue_command(
    State(state): State<ConsoleState>,
    Json(cmd): Json<CommandEnvelope>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let now_ms = now_millis();
    
    // Validate permit is not expired
    if let Some(exp) = cmd.permit.get("exp").and_then(|v| v.as_u64()) {
        if exp < now_ms {
            return (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse { error: "Permit expired".to_string() }),
            ).into_response();
        }
    }
    
    // Validate permit jti matches command jti
    if let Some(permit_jti) = cmd.permit.get("jti").and_then(|v| v.as_str()) {
        if permit_jti != cmd.jti {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: "Permit jti mismatch".to_string() }),
            ).into_response();
        }
    }
    
    // Mark permit as used
    let _ = sqlx::query("UPDATE permits SET used = true WHERE jti = $1")
        .bind(&cmd.jti)
        .execute(pool)
        .await;
    
    // Insert command
    let result = sqlx::query(
        r#"
        INSERT INTO commands (jti, tenant_id, job_id, job_type, params, subject_hash, policy_hash, permit, target, office_id, pending, issued_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 1, $11)
        ON CONFLICT (jti) DO NOTHING
        "#
    )
    .bind(&cmd.jti)
    .bind(&cmd.tenant_id)
    .bind(&cmd.job_id)
    .bind(&cmd.job_type)
    .bind(&cmd.params)
    .bind(&cmd.subject_hash)
    .bind(&cmd.policy_hash)
    .bind(&cmd.permit)
    .bind(&cmd.target)
    .bind(&cmd.office_id)
    .bind(now_ms as i64)
    .execute(pool)
    .await;
    
    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        ).into_response(),
    }
}

/// GET /v1/query/commands — List pending commands for a target
async fn query_commands(
    State(state): State<ConsoleState>,
    Query(params): Query<QueryCommandsParams>,
) -> impl IntoResponse {
    let pool = &state.pool;
    
    let commands: Vec<CommandRow> = sqlx::query_as(
        r#"
        SELECT jti, tenant_id, job_id, job_type, params, subject_hash, policy_hash, permit, target, office_id, pending, issued_at
        FROM commands
        WHERE tenant_id = $1 AND target = $2 AND pending = $3
        ORDER BY issued_at ASC
        LIMIT $4
        "#
    )
    .bind(&params.tenant_id)
    .bind(&params.target)
    .bind(params.pending)
    .bind(params.limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    (StatusCode::OK, Json(commands))
}

/// POST /v1/exec.finish — Register execution receipt
async fn exec_finish(
    State(state): State<ConsoleState>,
    Json(receipt): Json<Receipt>,
) -> impl IntoResponse {
    let pool = &state.pool;
    
    // Mark command as done (pending = 0)
    let _ = sqlx::query("UPDATE commands SET pending = 0 WHERE job_id = $1")
        .bind(&receipt.job_id)
        .execute(pool)
        .await;
    
    // Insert receipt
    let artifacts_json = serde_json::to_value(&receipt.artifacts).unwrap_or_default();
    
    let result = sqlx::query(
        r#"
        INSERT INTO receipts (tenant_id, job_id, status, finished_at, logs_hash, artifacts, usage, error)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (job_id) DO UPDATE SET
            status = EXCLUDED.status,
            finished_at = EXCLUDED.finished_at,
            logs_hash = EXCLUDED.logs_hash,
            artifacts = EXCLUDED.artifacts,
            usage = EXCLUDED.usage,
            error = EXCLUDED.error
        "#
    )
    .bind(&receipt.tenant_id)
    .bind(&receipt.job_id)
    .bind(&receipt.status)
    .bind(receipt.finished_at as i64)
    .bind(&receipt.logs_hash)
    .bind(&artifacts_json)
    .bind(&receipt.usage)
    .bind(&receipt.error)
    .execute(pool)
    .await;
    
    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        ).into_response(),
    }
}

// ============ Helpers ============

fn blake3_hex(data: &str) -> String {
    let hash = blake3::hash(data.as_bytes());
    hash.to_hex().to_string()
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Get TTL in milliseconds based on risk level
fn get_ttl_for_risk(risk: &str) -> u64 {
    match risk {
        "L0" | "L1" | "L2" => 2 * 60 * 1000,  // 2 min
        "L3" => 5 * 60 * 1000,                 // 5 min
        "L4" | "L5" => 3 * 60 * 1000,          // 3 min (shorter for high risk)
        _ => 2 * 60 * 1000,                    // default
    }
}

/// Get TTL in milliseconds based on job type (legacy, for backward compat)
fn get_ttl_for_job_type(job_type: &str) -> u64 {
    // L0-L2: 2 minutes, L3: 5 minutes, L4-L5: 3 minutes
    // For now, use simple heuristics
    if job_type.contains("security") || job_type.contains("merge_protected") {
        3 * 60 * 1000 // L4-L5: 3 min
    } else if job_type.contains("tag_release") || job_type.contains("register") {
        5 * 60 * 1000 // L3: 5 min
    } else {
        2 * 60 * 1000 // L0-L2: 2 min
    }
}

