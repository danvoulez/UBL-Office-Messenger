//! Button Provenance - Validates that button clicks come from real cards
//!
//! From the spec:
//! > Card Provenance Rule: If a user submits a card action, require that:
//! > 1. There exists a prior ledger event with that card_id
//! > 2. The button_id exists in that exact card payload
//! > 3. The action type matches the button's declared action
//!
//! This prevents:
//! - Forging approvals
//! - Replaying buttons from different jobs
//! - "Invented" actions not offered by the UI
//!
//! "No fake buttons. Ever."

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::job_executor::cards::{CardButton, CardAction};
use crate::{OfficeError, Result};

/// A validated button submission
#[derive(Debug, Clone)]
pub struct ValidatedAction {
    pub card_id: String,
    pub button_id: String,
    pub job_id: String,
    pub action: CardAction,
    /// Input provided by user (if any)
    pub input: Option<serde_json::Value>,
}

/// Button provenance validator
pub struct ProvenanceValidator {
    /// Cache of known cards: card_id -> (job_id, buttons)
    known_cards: RwLock<HashMap<String, (String, Vec<CardButton>)>>,
}

impl ProvenanceValidator {
    pub fn new() -> Self {
        Self {
            known_cards: RwLock::new(HashMap::new()),
        }
    }

    /// Register a card's buttons for validation
    pub async fn register_card(&self, card_id: &str, job_id: &str, buttons: Vec<CardButton>) {
        let mut cards = self.known_cards.write().await;
        cards.insert(card_id.to_string(), (job_id.to_string(), buttons));
    }

    /// Validate a button click
    pub async fn validate_action(
        &self,
        card_id: &str,
        button_id: &str,
        claimed_action: &CardAction,
        input: Option<serde_json::Value>,
    ) -> Result<ValidatedAction> {
        let cards = self.known_cards.read().await;

        // 1. Card must exist
        let (job_id, buttons) = cards.get(card_id)
            .ok_or_else(|| OfficeError::ProvenanceError(format!(
                "Unknown card_id: {}. Card not found in ledger.",
                card_id
            )))?;

        // 2. Button must exist in card
        let button = buttons.iter()
            .find(|b| b.button_id == button_id)
            .ok_or_else(|| OfficeError::ProvenanceError(format!(
                "Unknown button_id: {}. Button not found in card {}.",
                button_id, card_id
            )))?;

        // 3. Action type must match
        if !actions_match(&button.action, claimed_action) {
            return Err(OfficeError::ProvenanceError(format!(
                "Action mismatch. Button declares {:?} but received {:?}.",
                action_type_name(&button.action),
                action_type_name(claimed_action)
            )));
        }

        // 4. Validate input if required
        if button.requires_input && input.is_none() {
            return Err(OfficeError::ProvenanceError(format!(
                "Button {} requires input but none provided.",
                button_id
            )));
        }

        Ok(ValidatedAction {
            card_id: card_id.to_string(),
            button_id: button_id.to_string(),
            job_id: job_id.clone(),
            action: claimed_action.clone(),
            input,
        })
    }

    /// Clear a card from cache (after job completion)
    pub async fn clear_card(&self, card_id: &str) {
        let mut cards = self.known_cards.write().await;
        cards.remove(card_id);
    }

    /// Clear all cards for a job
    pub async fn clear_job_cards(&self, job_id: &str) {
        let mut cards = self.known_cards.write().await;
        cards.retain(|_, (jid, _)| jid != job_id);
    }
}

impl Default for ProvenanceValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two actions match (same type and job_id)
fn actions_match(expected: &CardAction, claimed: &CardAction) -> bool {
    use CardAction::*;
    
    match (expected, claimed) {
        (Approve { job_id: j1 }, Approve { job_id: j2 }) => j1 == j2,
        (Reject { job_id: j1, .. }, Reject { job_id: j2, .. }) => j1 == j2,
        (RequestChanges { job_id: j1 }, RequestChanges { job_id: j2 }) => j1 == j2,
        (ProvideInput { job_id: j1, .. }, ProvideInput { job_id: j2, .. }) => j1 == j2,
        (Acknowledge { job_id: j1 }, Acknowledge { job_id: j2 }) => j1 == j2,
        (Dispute { job_id: j1, .. }, Dispute { job_id: j2, .. }) => j1 == j2,
        (Cancel { job_id: j1 }, Cancel { job_id: j2 }) => j1 == j2,
        (ChatAsk { job_id: j1, .. }, ChatAsk { job_id: j2, .. }) => j1 == j2,
        _ => false,
    }
}

/// Get the type name of an action for error messages
fn action_type_name(action: &CardAction) -> &'static str {
    use CardAction::*;
    match action {
        Approve { .. } => "job.approve",
        Reject { .. } => "job.reject",
        RequestChanges { .. } => "job.request_changes",
        ProvideInput { .. } => "job.provide_input",
        Acknowledge { .. } => "job.ack",
        Dispute { .. } => "job.dispute",
        Cancel { .. } => "job.cancel",
        ChatAsk { .. } => "chat.ask",
    }
}

/// Provenance check result for policy enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceCheck {
    pub valid: bool,
    pub card_id: String,
    pub button_id: String,
    pub job_id: Option<String>,
    pub error: Option<String>,
}

impl ProvenanceCheck {
    pub fn success(card_id: &str, button_id: &str, job_id: &str) -> Self {
        Self {
            valid: true,
            card_id: card_id.to_string(),
            button_id: button_id.to_string(),
            job_id: Some(job_id.to_string()),
            error: None,
        }
    }

    pub fn failure(card_id: &str, button_id: &str, error: &str) -> Self {
        Self {
            valid: false,
            card_id: card_id.to_string(),
            button_id: button_id.to_string(),
            job_id: None,
            error: Some(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job_executor::cards::ButtonStyle;

    #[tokio::test]
    async fn test_valid_button_click() {
        let validator = ProvenanceValidator::new();
        
        let buttons = vec![
            CardButton {
                button_id: "btn_approve_123".to_string(),
                label: "Approve".to_string(),
                action: CardAction::Approve { job_id: "job_123".to_string() },
                style: Some(ButtonStyle::Primary),
                requires_input: false,
                confirm: None,
            },
        ];
        
        validator.register_card("card_abc", "job_123", buttons).await;
        
        let result = validator.validate_action(
            "card_abc",
            "btn_approve_123",
            &CardAction::Approve { job_id: "job_123".to_string() },
            None,
        ).await;
        
        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.job_id, "job_123");
    }

    #[tokio::test]
    async fn test_unknown_card() {
        let validator = ProvenanceValidator::new();
        
        let result = validator.validate_action(
            "fake_card",
            "fake_button",
            &CardAction::Approve { job_id: "job_123".to_string() },
            None,
        ).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown card_id"));
    }

    #[tokio::test]
    async fn test_action_mismatch() {
        let validator = ProvenanceValidator::new();
        
        let buttons = vec![
            CardButton {
                button_id: "btn_approve_123".to_string(),
                label: "Approve".to_string(),
                action: CardAction::Approve { job_id: "job_123".to_string() },
                style: Some(ButtonStyle::Primary),
                requires_input: false,
                confirm: None,
            },
        ];
        
        validator.register_card("card_abc", "job_123", buttons).await;
        
        // Try to use Approve button for Reject action
        let result = validator.validate_action(
            "card_abc",
            "btn_approve_123",
            &CardAction::Reject { job_id: "job_123".to_string(), reason_code: None },
            None,
        ).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Action mismatch"));
    }

    #[tokio::test]
    async fn test_missing_required_input() {
        let validator = ProvenanceValidator::new();
        
        let buttons = vec![
            CardButton {
                button_id: "btn_input_123".to_string(),
                label: "Provide info".to_string(),
                action: CardAction::ProvideInput { job_id: "job_123".to_string(), input_schema: None },
                style: Some(ButtonStyle::Secondary),
                requires_input: true,
                confirm: None,
            },
        ];
        
        validator.register_card("card_abc", "job_123", buttons).await;
        
        // Try without input
        let result = validator.validate_action(
            "card_abc",
            "btn_input_123",
            &CardAction::ProvideInput { job_id: "job_123".to_string(), input_schema: None },
            None, // No input!
        ).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("requires input"));
    }
}

