//! Console API v1.1 — ADR-UBL-Console-001
//!
//! SECURITY-HARDENED VERSION with:
//! - Real WebAuthn step-up for L4/L5
//! - Permit binding_hash cryptographic commitment
//! - Single-use atomic permit consumption
//! - Runner signature verification on receipts
//!
//! Endpoints:
//! - POST /v1/policy/permit      → Emit Permit (step-up required for L4/L5)
//! - POST /v1/id/stepup/begin    → Begin step-up (returns WebAuthn challenge)
//! - POST /v1/commands/issue     → Register Command (atomic single-use)
//! - GET  /v1/query/commands     → List pending commands for Runner
//! - POST /v1/exec.finish        → Register Receipt (runner signature required)

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction, Row};
use webauthn_rs::prelude::*;

use crate::crypto;
use crate::webauthn_store;

// =============================================================================
// STATE
// =============================================================================

#[derive(Clone)]
pub struct ConsoleState {
    pub pool: PgPool,
    pub webauthn: Webauthn,
}

// =============================================================================
// ROUTES
// =============================================================================

pub fn routes(pool: PgPool, webauthn: Webauthn) -> Router {
    let state = ConsoleState { pool, webauthn };
    Router::new()
        .route("/v1/policy/permit", post(issue_permit))
        .route("/v1/id/stepup/begin", post(stepup_begin))
        .route("/v1/commands/issue", post(issue_command))
        .route("/v1/query/commands", get(query_commands))
        .route("/v1/exec.finish", post(exec_finish))
        .with_state(state)
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct PermitRequest {
    pub office: String,
    pub action: String,
    pub target: String,
    pub args: serde_json::Value,
    pub plan: serde_json::Value,
    #[serde(default = "default_risk")]
    pub risk: String,
    /// WebAuthn assertion for L4/L5 step-up (from navigator.credentials.get)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stepup_assertion: Option<serde_json::Value>,
}

fn default_risk() -> String {
    "L0".to_string()
}

#[derive(Debug, Serialize)]
pub struct Permit {
    pub jti: String,
    pub office: String,
    pub action: String,
    pub target: String,
    pub args: serde_json::Value,
    pub risk: String,
    pub plan_hash: String,
    pub nonce: String,
    pub issued_at_ms: i64,
    pub exp_ms: i64,
    pub binding_hash: String,
    pub approver: String,
    pub sig: String,
}

#[derive(Debug, Serialize)]
pub struct PermitResponse {
    pub permit: Permit,
    pub allowed: bool,
}

#[derive(Debug, Deserialize)]
pub struct StepUpBeginRequest {
    pub user_id: String,
    pub binding_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandIssueRequest {
    pub permit_jti: String,
}

#[derive(Debug, Serialize)]
pub struct CommandIssued {
    pub command_id: String,
    pub pending: bool,
}

#[derive(Debug, Deserialize)]
pub struct QueryCommandsParams {
    pub target: String,
    #[serde(default = "default_pending")]
    pub pending: bool,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_pending() -> bool { true }
fn default_limit() -> i32 { 10 }

#[derive(Debug, Serialize)]
pub struct CommandRow {
    pub command_id: String,
    pub permit_jti: String,
    pub office: String,
    pub action: String,
    pub target: String,
    pub args: serde_json::Value,
    pub risk: String,
    pub plan_hash: String,
    pub binding_hash: String,
    pub pending: bool,
    pub created_at_ms: i64,
}

#[derive(Debug, Deserialize)]
pub struct ExecFinishRequest {
    pub command_id: String,
    pub runner_id: String,
    pub status: String, // "OK" | "ERROR"
    pub logs_hash: String,
    pub ret: serde_json::Value,
    pub sig_runner: String, // "ed25519:<base64url>"
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /v1/id/stepup/begin — Begin step-up authentication
/// Returns WebAuthn challenge tied to a binding_hash
async fn stepup_begin(
    State(state): State<ConsoleState>,
    Json(req): Json<StepUpBeginRequest>,
) -> Result<Json<webauthn_store::StepUpBeginResponse>, (StatusCode, String)> {
    let inner_req = webauthn_store::StepUpBeginRequest {
        user_id: req.user_id,
        binding_hash: req.binding_hash,
    };
    let response = webauthn_store::begin_stepup(&state.pool, &state.webauthn, &inner_req).await?;
    Ok(Json(response))
}

/// POST /v1/policy/permit — Issue a Permit
/// SECURITY: L4/L5 require WebAuthn step-up with binding_hash verification
async fn issue_permit(
    State(state): State<ConsoleState>,
    Json(req): Json<PermitRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let now_ms = now_millis() as i64;

    // 1. Compute plan_hash
    let plan_hash = crypto::canonical_plan_hash(&req.plan);

    // 2. Generate nonce
    let nonce_bytes = crypto::rand_bytes_16();
    let nonce = URL_SAFE_NO_PAD.encode(&nonce_bytes);

    // 3. Compute TTL based on risk
    let ttl_ms = get_ttl_for_risk(&req.risk);
    let exp_ms = now_ms + ttl_ms;

    // 4. Compute binding_hash (cryptographic commitment)
    let binding_hash = crypto::permit_binding_hash(
        &req.office,
        &req.action,
        &req.target,
        &req.args,
        &req.risk,
        &plan_hash,
        &nonce,
        exp_ms,
    );

    // 5. Step-up required for L4/L5
    let needs_stepup = req.risk == "L4" || req.risk == "L5";
    let approver: String;

    if needs_stepup {
        let assertion = match &req.stepup_assertion {
            Some(a) if !a.is_null() => a,
            _ => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: format!(
                            "StepUpRequired: {} operations require WebAuthn assertion. \
                            First call /v1/id/stepup/begin with binding_hash={}",
                            req.risk, binding_hash
                        ),
                    }),
                )
                    .into_response();
            }
        };

        // Verify step-up assertion against binding_hash
        match webauthn_store::verify_stepup_assertion(
            pool,
            &state.webauthn,
            assertion,
            &binding_hash,
        )
        .await
        {
            Ok(true) => {
                approver = webauthn_store::extract_approver_id(assertion)
                    .unwrap_or_else(|| "webauthn:unknown".into());
                tracing::info!(
                    risk = %req.risk,
                    approver = %approver,
                    binding_hash = %binding_hash,
                    "✅ Step-up verified for high-risk permit"
                );
            }
            Ok(false) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "StepUpFailed: assertion verification returned false".into(),
                    }),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::warn!(error = %e, "Step-up verification failed");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: format!("StepUpFailed: {}", e),
                    }),
                )
                    .into_response();
            }
        }
    } else {
        // L0-L3: session-based (would check Authorization header)
        approver = "session:default".into();
    }

    // 6. Sign the permit with admin key
    let sig_bytes = crypto::sign_admin_permit(binding_hash.as_bytes());
    let sig = format!("ed25519:{}", URL_SAFE_NO_PAD.encode(&sig_bytes));

    // 7. Generate JTI and persist
    let jti = crypto::uuid_v4();

    let insert_result = sqlx::query(
        r#"
        INSERT INTO console_permits
          (jti, office, action, target, args_json, risk, plan_hash, nonce, issued_at_ms, exp_ms, binding_hash, approver, sig, used)
        VALUES
          ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, false)
        "#,
    )
    .bind(&jti)
    .bind(&req.office)
    .bind(&req.action)
    .bind(&req.target)
    .bind(&req.args)
    .bind(&req.risk)
    .bind(&plan_hash)
    .bind(&nonce)
    .bind(now_ms)
    .bind(exp_ms)
    .bind(&binding_hash)
    .bind(&approver)
    .bind(&sig)
    .execute(pool)
    .await;

    if let Err(e) = insert_result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("DB error: {}", e),
            }),
        )
            .into_response();
    }

    // 8. Return permit
    let permit = Permit {
        jti,
        office: req.office,
        action: req.action,
        target: req.target,
        args: req.args,
        risk: req.risk,
        plan_hash,
        nonce,
        issued_at_ms: now_ms,
        exp_ms,
        binding_hash,
        approver,
        sig,
    };

    (StatusCode::OK, Json(PermitResponse { permit, allowed: true })).into_response()
}

/// POST /v1/commands/issue — Atomically consume permit and create command
async fn issue_command(
    State(state): State<ConsoleState>,
    Json(req): Json<CommandIssueRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let now_ms = now_millis() as i64;

    // Begin transaction for atomic single-use
    let mut tx: Transaction<Postgres> = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response();
        }
    };

    // Lock and fetch permit (FOR UPDATE)
    let permit_row = sqlx::query(
        r#"
        SELECT jti, office, action, target, args_json, risk, plan_hash, nonce,
               issued_at_ms, exp_ms, binding_hash, approver, sig, used
        FROM console_permits
        WHERE jti = $1
        FOR UPDATE
        "#,
    )
    .bind(&req.permit_jti)
    .fetch_optional(&mut *tx)
    .await;

    let row = match permit_row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: "PermitNotFound".into() }),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response();
        }
    };

    // Extract fields
    let used: bool = Row::get(&row, "used");
    let exp_ms: i64 = Row::get(&row, "exp_ms");
    let binding_hash: String = Row::get(&row, "binding_hash");
    let sig: String = Row::get(&row, "sig");
    let office: String = Row::get(&row, "office");
    let action: String = Row::get(&row, "action");
    let target: String = Row::get(&row, "target");
    let args_json: serde_json::Value = Row::get(&row, "args_json");
    let risk: String = Row::get(&row, "risk");
    let plan_hash: String = Row::get(&row, "plan_hash");

    // Validate: not used
    if used {
        return (
            StatusCode::CONFLICT,
            Json(ErrorResponse { error: "PermitAlreadyUsed".into() }),
        )
            .into_response();
    }

    // Validate: not expired
    if now_ms > exp_ms {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse { error: "PermitExpired".into() }),
        )
            .into_response();
    }

    // Validate: signature
    if let Err(e) = crypto::verify_admin_permit_sig(binding_hash.as_bytes(), &sig) {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse { error: format!("PermitSigInvalid: {}", e) }),
        )
            .into_response();
    }

    // Mark permit as used
    if let Err(e) = sqlx::query("UPDATE console_permits SET used = true WHERE jti = $1")
        .bind(&req.permit_jti)
        .execute(&mut *tx)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response();
    }

    // Create command
    let command_id = crypto::uuid_v4();

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO console_commands
          (command_id, permit_jti, office, action, target, args_json, risk, plan_hash, binding_hash, pending, created_at_ms)
        VALUES
          ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10)
        "#,
    )
    .bind(&command_id)
    .bind(&req.permit_jti)
    .bind(&office)
    .bind(&action)
    .bind(&target)
    .bind(&args_json)
    .bind(&risk)
    .bind(&plan_hash)
    .bind(&binding_hash)
    .bind(now_ms)
    .execute(&mut *tx)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response();
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response();
    }

    (StatusCode::OK, Json(CommandIssued { command_id, pending: true })).into_response()
}

/// GET /v1/query/commands — List pending commands for a target (Runner pulls)
async fn query_commands(
    State(state): State<ConsoleState>,
    Query(params): Query<QueryCommandsParams>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let rows = sqlx::query(
        r#"
        SELECT command_id, permit_jti, office, action, target, args_json, risk, plan_hash, binding_hash, pending, created_at_ms
        FROM console_commands
        WHERE target = $1 AND pending = $2
        ORDER BY created_at_ms ASC
        LIMIT $3
        "#,
    )
    .bind(&params.target)
    .bind(params.pending)
    .bind(params.limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let commands: Vec<CommandRow> = rows
        .into_iter()
        .map(|row| CommandRow {
            command_id: Row::get(&row, "command_id"),
            permit_jti: Row::get(&row, "permit_jti"),
            office: Row::get(&row, "office"),
            action: Row::get(&row, "action"),
            target: Row::get(&row, "target"),
            args: Row::get(&row, "args_json"),
            risk: Row::get(&row, "risk"),
            plan_hash: Row::get(&row, "plan_hash"),
            binding_hash: Row::get(&row, "binding_hash"),
            pending: Row::get(&row, "pending"),
            created_at_ms: Row::get(&row, "created_at_ms"),
        })
        .collect();

    (StatusCode::OK, Json(commands))
}

/// POST /v1/exec.finish — Register execution receipt (must be signed by Runner)
async fn exec_finish(
    State(state): State<ConsoleState>,
    Json(req): Json<ExecFinishRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let now_ms = now_millis() as i64;

    // Fetch command
    let cmd_row = sqlx::query(
        r#"
        SELECT command_id, permit_jti, binding_hash, pending
        FROM console_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&req.command_id)
    .fetch_optional(pool)
    .await;

    let cmd = match cmd_row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: "CommandNotFound".into() }),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response();
        }
    };

    let pending: bool = Row::get(&cmd, "pending");
    if !pending {
        return (
            StatusCode::CONFLICT,
            Json(ErrorResponse { error: "CommandAlreadyFinished".into() }),
        )
            .into_response();
    }

    let permit_jti: String = Row::get(&cmd, "permit_jti");
    let binding_hash: String = Row::get(&cmd, "binding_hash");

    // Build receipt payload for signature verification
    let receipt_payload = serde_json::json!({
        "command_id": req.command_id,
        "permit_jti": permit_jti,
        "binding_hash": binding_hash,
        "runner_id": req.runner_id,
        "status": req.status,
        "logs_hash": req.logs_hash,
        "ret": req.ret,
    });

    // Canonicalize for signature
    let receipt_bytes = match crate::crypto::ubl_atom_compat::canonicalize(&receipt_payload) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: format!("CanonicalizeError: {}", e) }),
            )
                .into_response();
        }
    };

    // Verify runner signature
    if let Err(e) = crypto::verify_runner_sig(pool, &req.runner_id, &receipt_bytes, &req.sig_runner).await {
        tracing::warn!(error = %e, runner_id = %req.runner_id, "Runner signature verification failed");
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse { error: format!("RunnerSigInvalid: {}", e) }),
        )
            .into_response();
    }

    // Begin transaction
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response();
        }
    };

    // Mark command as done
    if let Err(e) = sqlx::query("UPDATE console_commands SET pending = false WHERE command_id = $1")
        .bind(&req.command_id)
        .execute(&mut *tx)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response();
    }

    // Insert receipt
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO console_receipts
          (command_id, permit_jti, runner_id, status, logs_hash, ret_json, sig_runner, finished_at_ms)
        VALUES
          ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(&req.command_id)
    .bind(&permit_jti)
    .bind(&req.runner_id)
    .bind(&req.status)
    .bind(&req.logs_hash)
    .bind(&req.ret)
    .bind(&req.sig_runner)
    .bind(now_ms)
    .execute(&mut *tx)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response();
    }

    // Commit
    if let Err(e) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response();
    }

    tracing::info!(
        command_id = %req.command_id,
        runner_id = %req.runner_id,
        status = %req.status,
        "✅ Execution receipt recorded"
    );

    (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
}

// =============================================================================
// HELPERS
// =============================================================================

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn get_ttl_for_risk(risk: &str) -> i64 {
    match risk {
        "L0" | "L1" => 10 * 60 * 1000,     // 10 min
        "L2" => 5 * 60 * 1000,              // 5 min
        "L3" => 3 * 60 * 1000,              // 3 min
        "L4" | "L5" => 90 * 1000,           // 90 sec (high risk = short lived)
        _ => 5 * 60 * 1000,                 // default 5 min
    }
}
