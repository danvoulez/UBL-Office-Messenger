//! Workspace routes (ws/test, ws/build)
//! Prompt 1: Office front-door - workspace operations

use axum::{extract::State, response::IntoResponse, Json, http::{StatusCode, HeaderMap}};
use serde_json::json;
use urlencoding;

use crate::types::{WsTestBody, WsBuildBody};
use crate::asc::validate_sid_and_asc;
use crate::ubl_client::UblClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct OfficeState {
    pub ubl_base: String, // ex: http://lab256.ubl.agency
    pub ubl_client: Arc<UblClient>,
}

pub fn router(state: OfficeState) -> axum::Router {
    axum::Router::new()
        .route("/office/ws/test", axum::routing::post(ws_test))
        .route("/office/ws/build", axum::routing::post(ws_build))
        .with_state(state)
}

pub async fn ws_test(
    State(state): State<OfficeState>,
    headers: HeaderMap,
    Json(body): Json<WsTestBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let container_id = format!("workspace://{}/{}", body.tenant, body.workspace);
    let intent_class = "Entropy";
    validate_sid_and_asc(&headers, &container_id, intent_class, 1).await?;

    let atom = json!({
        "kind": "ws/test/request",
        "tenant": body.tenant,
        "workspace": body.workspace,
        "repo": body.repo,
        "sha": body.sha,
        "suite": body.suite,
        "limits": body.limits
    });

    let link_hash = commit(&state.ubl_client, &container_id, intent_class, 1, &atom).await?;
    
    if body.wait.unwrap_or(true) {
        let receipt = wait_for_receipt(&state.ubl_base, &container_id, &link_hash).await?;
        return Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash, "receipt": receipt}))));
    }
    
    Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash}))))
}

pub async fn ws_build(
    State(state): State<OfficeState>,
    headers: HeaderMap,
    Json(body): Json<WsBuildBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let container_id = format!("workspace://{}/{}", body.tenant, body.workspace);
    let intent_class = "Entropy";
    validate_sid_and_asc(&headers, &container_id, intent_class, 1).await?;

    let atom = json!({
        "kind": "ws/build/request",
        "tenant": body.tenant,
        "workspace": body.workspace,
        "repo": body.repo,
        "sha": body.sha,
        "target": body.target,
        "limits": body.limits
    });

    let link_hash = commit(&state.ubl_client, &container_id, intent_class, 1, &atom).await?;
    
    if body.wait.unwrap_or(true) {
        let receipt = wait_for_receipt(&state.ubl_base, &container_id, &link_hash).await?;
        return Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash, "receipt": receipt}))));
    }
    
    Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash}))))
}

// ---- helpers (exported for use in deploy.rs) ----
pub(crate) async fn commit(
    ubl_client: &UblClient,
    container_id: &str,
    intent_class: &str,
    delta: i128,
    atom_json: &serde_json::Value,
) -> Result<String, (StatusCode, String)> {
    // 1) canonicalize + hash (JSON✯Atomic)
    let canonical = ubl_atom::canonicalize(atom_json)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("atom error: {e}")))?;
    let atom_hash = ubl_kernel::hash_atom(&canonical); // BLAKE3(canonical_bytes), sem domain tag

    // 2) Use UblClient's commit_atom which handles signing automatically
    let response = ubl_client.commit_atom(
        container_id,
        atom_json,
        intent_class,
        delta as i64,
    ).await
    .map_err(|e| (StatusCode::BAD_GATEWAY, format!("gateway error: {e}")))?;
    
    if !response.ok {
        return Err((StatusCode::BAD_GATEWAY, "Commit failed".to_string()));
    }
    
    Ok(atom_hash)
}

pub async fn wait_for_receipt(
    _ubl_base: &str, // Not used anymore, we use UblEndpoint::from_env()
    container_id: &str,
    trigger_link_hash: &str,
) -> Result<serde_json::Value, (StatusCode, String)> {
    // Prompt 3: Use UblEndpoint to support Unix Socket
    use crate::http_unix::{UblEndpoint, body_to_bytes};
    
    // Estratégia simples: poll entries recentes do ledger e procurar ws/receipt com trigger == link_hash
    // (pode trocar por SSE client conforme infra disponível)
    for _ in 0..120 {
        let path = format!("/ledger/{}/latest?limit=50", urlencoding::encode(container_id));
        let ubl = UblEndpoint::from_env();
        
        let resp = ubl.get(&path).await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("tail error: {e}")))?;
        
        let status_code = resp.status().as_u16();
        if status_code < 200 || status_code >= 300 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }
        
        let body_bytes = body_to_bytes(resp.into_body()).await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("read body error: {e}")))?;
        let entries: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("parse tail error: {e}")))?;
        
        if let Some(arr) = entries.as_array() {
            for e in arr {
                if let Some(atom) = e.get("atom") {
                    if atom.get("kind") == Some(&serde_json::Value::String("ws/receipt".into()))
                        && atom.get("trigger") == Some(&serde_json::Value::String(trigger_link_hash.into()))
                    {
                        return Ok(atom.clone());
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Err((StatusCode::GATEWAY_TIMEOUT, "receipt timeout".into()))
}

