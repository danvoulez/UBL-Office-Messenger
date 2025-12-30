//! ASC (Authorization Scope Certificate) validation
//! Prompt 1: Checagem leve de SID/ASC

use axum::http::StatusCode;
use serde::Deserialize;

#[derive(Clone)]
pub struct Asc {
    pub asc_id: String,
    pub scope_container: String,
    pub intent_classes: Vec<String>,
    pub max_delta: i128,
}

pub async fn validate_sid_and_asc(
    headers: &axum::http::HeaderMap,
    target_container: &str,
    intent_class: &str,
    delta: i128,
) -> Result<Asc, (StatusCode, String)> {
    // Exemplo: pegar SID/Bearer + X-UBL-ASC dos headers
    let _sid = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let asc_id = headers
        .get("x-ubl-asc")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    if asc_id.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "ASC missing".into()));
    }

    // TODO: consultar id_session + id_asc no UBL (ou cache local) e validar scope:
    // - container_id prefix
    // - intent_class permitida
    // - max_delta >= delta
    // - janela de tempo

    Ok(Asc {
        asc_id: asc_id.to_string(),
        scope_container: target_container.to_string(),
        intent_classes: vec![intent_class.to_string()],
        max_delta: 0,
    })
}


