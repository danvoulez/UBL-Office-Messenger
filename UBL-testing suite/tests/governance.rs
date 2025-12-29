//! Governance Tests
//! Tests constitution enforcement, sanity checks, dreaming cycle, and simulation

use office::{
    governance: :{
        constitution: :{OfficeConstitution, ModeConfig, PreFlightConfig},
        sanity_check::{SanityChecker, Claim, Fact},
        dreaming: :{DreamingCycle, DreamingConfig, DreamingResult},
        simulation::{Simulator, SimulationRequest, SimulationResult, ActionOutcome},
    },
    entity::Entity,
    session: :{Handover, HandoverBuilder},
};
use uuid::Uuid;
use time::OffsetDateTime;

#[tokio::test]
async fn test_default_constitution() {
    let constitution = OfficeConstitution:: default();
    
    assert_eq!(constitution.version, "1.0");
    assert_eq!(constitution.precedence, "UBL > Office");
    assert! (!constitution.core_directive.is_empty());
}

#[tokio::test]
async fn test_constitution_mode_config() {
    let mut constitution = OfficeConstitution::default();
    
    // Configure Work mode
    constitution.allow_modes. insert(
        "Work".to_string(),
        ModeConfig {
            max_risk:  0.8,
            require_step_up: false,
        },
    );
    
    assert!(constitution.allow_modes.contains_key("Work"));
    assert_eq!(constitution.allow_modes["Work"].max_risk, 0.8);
}

#[tokio::test]
async fn test_constitution_denylists() {
    let mut constitution = OfficeConstitution:: default();
    
    constitution.denylists.job_types.push("dangerous. operation".to_string());
    constitution.denylists.targets.push("production-db".to_string());
    
    assert!(constitution.denylists.job_types.contains(&"dangerous.operation".to_string()));
    assert!(constitution.denylists.targets.contains(&"production-db".to_string()));
}

#[tokio:: test]
async fn test_constitution_pre_flight() {
    let mut constitution = OfficeConstitution::default();
    
    constitution.pre_flight. require_diff_for. push("deploy".to_string());
    
    assert!(constitution.pre_flight.require_diff_for.contains(&"deploy".to_string()));
}

#[tokio:: test]
async fn test_sanity_check_no_discrepancies() {
    let checker = SanityChecker::new();
    
    let handover_text = "Completed task successfully.  All tests passing.";
    let facts = vec![
        Fact {
            description: "Tests passed".to_string(),
            timestamp: OffsetDateTime::now_utc(),
            source: "CI system".to_string(),
        },
    ];
    
    let result = checker.check(handover_text, &facts);
    
    assert!(result.is_ok());
    let governance_notes = result.unwrap();
    assert_eq!(governance_notes.len(), 0); // No discrepancies
}

#[tokio::test]
async fn test_sanity_check_with_discrepancy() {
    let checker = SanityChecker::new();
    
    let handover_text = "Deployment failed due to connection timeout. ";
    let facts = vec![
        Fact {
            description: "Deployment succeeded".to_string(),
            timestamp: OffsetDateTime::now_utc(),
            source: "Deploy system".to_string(),
        },
    ];
    
    let result = checker.check(handover_text, &facts);
    
    assert!(result.is_ok());
    let governance_notes = result.unwrap();
    assert!(governance_notes.len() > 0); // Discrepancy detected
    assert!(governance_notes[0].contains("GOVERNANCE NOTE"));
}

#[tokio::test]
async fn test_claim_extraction() {
    let checker = SanityChecker::new();
    
    let text = "The system is experiencing critical failures.  Performance is degraded.";
    let claims = checker.extract_claims(text);
    
    assert!(claims.len() > 0);
    assert!(claims. iter().any(|c| c.text. contains("critical") || c.text.contains("failures")));
}

#[tokio::test]
async fn test_sentiment_estimation() {
    let checker = SanityChecker::new();
    
    let positive_text = "Everything is working great! ";
    let negative_text = "System failed catastrophically.";
    
    let positive_sentiment = checker.estimate_sentiment(positive_text);
    let negative_sentiment = checker.estimate_sentiment(negative_text);
    
    assert!(positive_sentiment > 0.0);
    assert!(negative_sentiment < 0.0);
}

#[tokio::test]
async fn test_dreaming_config_defaults() {
    let config = DreamingConfig::default();
    
    assert_eq!(config. session_threshold, 50);
    assert_eq!(config. time_threshold_hours, 24);
    assert!(config.archive_resolved);
}

#[tokio::test]
async fn test_dreaming_cycle_trigger_conditions() {
    let config = DreamingConfig:: default();
    let mut entity = Entity::new(
        format!("entity_{}", Uuid:: new_v4()),
        "Test". to_string(),
        office::entity::EntityType:: Autonomous,
    );
    
    // Not due yet
    entity.total_sessions = 10;
    assert!(!config.is_due(&entity));
    
    // Due by session threshold
    entity.total_sessions = 51;
    assert!(config.is_due(&entity));
}

#[tokio::test]
async fn test_dreaming_result_structure() {
    let result = DreamingResult {
        entity_id: format! ("entity_{}", Uuid::new_v4()),
        started_at: OffsetDateTime::now_utc(),
        ended_at: OffsetDateTime::now_utc(),
        events_processed: 100,
        events_archived: 20,
        anxieties_cleared: vec! ["Issue #123 resolved".to_string()],
        patterns:  vec! ["Frequently uses Work sessions".to_string()],
        new_baseline: "Updated baseline narrative".to_string(),
        syntheses_created: 2,
        errors:  vec![],
    };
    
    assert_eq!(result.events_processed, 100);
    assert_eq!(result.events_archived, 20);
    assert_eq!(result.anxieties_cleared.len(), 1);
}

#[tokio::test]
async fn test_simulation_request() {
    let request = SimulationRequest {
        action: "deploy_to_production".to_string(),
        context: serde_json::json!({
            "target": "api-server",
            "version": "v1.2.3"
        }),
        risk_score: 0.8,
    };
    
    assert_eq!(request.action, "deploy_to_production");
    assert_eq!(request.risk_score, 0.8);
}

#[tokio:: test]
async fn test_simulation_outcome_generation() {
    let simulator = Simulator::new();
    
    let request = SimulationRequest {
        action: "test_action".to_string(),
        context: serde_json:: json!({}),
        risk_score: 0.5,
    };
    
    let result = simulator.simulate(&request);
    
    assert!(result.outcomes.len() > 0);
    assert!(result.recommendation.is_some());
}

#[tokio::test]
async fn test_simulation_high_risk_requires_confirmation() {
    let simulator = Simulator::new();
    
    let request = SimulationRequest {
        action: "dangerous_action".to_string(),
        context: serde_json:: json!({}),
        risk_score: 0.9,
    };
    
    let result = simulator.simulate(&request);
    
    // High risk should recommend SeekConfirmation or Abandon
    assert!(matches!(
        result.recommendation. as_ref().unwrap().as_str(),
        "SeekConfirmation" | "Abandon"
    ));
}

#[tokio::test]
async fn test_simulation_low_risk_proceeds() {
    let simulator = Simulator::new();
    
    let request = SimulationRequest {
        action: "safe_action".to_string(),
        context: serde_json::json!({}),
        risk_score:  0.1,
    };
    
    let result = simulator.simulate(&request);
    
    // Low risk should recommend Proceed
    assert_eq!(result.recommendation.unwrap(), "Proceed");
}

#[tokio::test]
async fn test_simulation_modification_suggestions() {
    let simulator = Simulator::new();
    
    let request = SimulationRequest {
        action: "risky_action".to_string(),
        context: serde_json::json!({}),
        risk_score: 0.6,
    };
    
    let result = simulator.simulate(&request);
    
    // Should have modifications
    assert!(result.modifications. len() > 0);
}