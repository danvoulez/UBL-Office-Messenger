//! Test fixtures and data builders

use serde_json::Value;
use uuid::Uuid;

/// Generate a test conversation ID
pub fn test_conversation_id() -> String {
    format!("conv_{}", Uuid::new_v4())
}

/// Generate a test message ID
pub fn test_message_id() -> String {
    format!("msg_{}", Uuid::new_v4())
}

/// Generate a test job ID
pub fn test_job_id() -> String {
    format!("job_{}", Uuid::new_v4())
}

/// Generate a test entity ID
pub fn test_entity_id() -> String {
    format!("entity_{}", Uuid::new_v4())
}

/// Create a test message payload
pub fn test_message(content: &str) -> Value {
    serde_json::json!({
        "content": content,
        "message_type": "text",
    })
}

/// Create a test job action payload
pub fn test_job_action(action_type: &str, card_id: &str, button_id: &str) -> Value {
    serde_json::json!({
        "action_type": action_type,
        "card_id": card_id,
        "button_id": button_id,
    })
}

