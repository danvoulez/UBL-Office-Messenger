//! Deploy routes
//! Prompt 1: Office front-door - deployment operations

use axum::{extract::State, response::IntoResponse, Json, http::{StatusCode, HeaderMap}};
use serde_json::json;

use crate::types::DeployBody;
use crate::asc::validate_sid_and_asc;
use crate::ubl_client::UblClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct OfficeState {
    pub ubl_base: String,
    pub ubl_client: Arc<UblClient>,
}

pub fn router(state: OfficeState) -> axum::Router {
    axum::Router::new()
        .route("/office/deploy", axum::routing::post(deploy))
        .with_state(state)
}

pub async fn deploy(
    State(state): State<OfficeState>,
    headers: HeaderMap,
    Json(body): Json<DeployBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let container_id = format!("deploy://{}/env/{}", body.tenant, body.env);
    let intent_class = "Entropy";
    validate_sid_and_asc(&headers, &container_id, intent_class, 1).await?;

    let atom = json!({
        "kind": "deploy/request",
        "tenant": body.tenant,
        "app": body.app,
        "env": body.env,
        "image_digest": body.image_digest,
        "strategy": body.strategy,
        "desired_replicas": body.desired_replicas
    });

    let link_hash = super::ws::commit(&state.ubl_client, &container_id, intent_class, 1, &atom).await?;
    
    if body.wait.unwrap_or(true) {
        let receipt = super::ws::wait_for_receipt(&state.ubl_base, &container_id, &link_hash).await?;
        return Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash, "receipt": receipt}))));
    }
    
    Ok((StatusCode::ACCEPTED, Json(json!({"link_hash": link_hash}))))
}

