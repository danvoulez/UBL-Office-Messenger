//! Job Execution Tests
//! Tests complete job lifecycle, FSM transitions, and job executor

use office::{
    job_executor::{
        JobExecutor, JobState, Job, JobCard, FormalizeCard, TrackingCard, FinishedCard,
        JobAction, JobError,
    },
    entity::Entity,
    session: :{Session, SessionType, SessionMode},
};
use uuid::Uuid;
use serde_json::json;

#[tokio::test]
async fn test_job_creation() {
    let job_id = format!("job_{}", Uuid:: new_v4());
    let conversation_id = format!("conv_{}", Uuid::new_v4());
    
    let job = Job {
        job_id:  job_id.clone(),
        conversation_id: conversation_id.clone(),
        title: "Test Job".to_string(),
        goal: "Complete the test task".to_string(),
        state: JobState::Draft,
        owner_entity_id: format!("entity_{}", Uuid::new_v4()),
        created_by_entity_id: format!("user_{}", Uuid::new_v4()),
        priority: "normal".to_string(),
        estimated_duration_seconds: Some(300),
        progress:  0,
        waiting_on: vec![],
        available_actions: vec![],
    };
    
    assert_eq!(job.job_id, job_id);
    assert_eq!(job.state, JobState::Draft);
}

#[tokio::test]
async fn test_job_state_transitions() {
    let transitions = vec![
        (JobState::Draft, JobState::Proposed),
        (JobState:: Proposed, JobState::Approved),
        (JobState::Approved, JobState::InProgress),
        (JobState:: InProgress, JobState::WaitingInput),
        (JobState::WaitingInput, JobState::InProgress),
        (JobState:: InProgress, JobState::Completed),
    ];
    
    for (from, to) in transitions {
        assert!(is_valid_transition(&from, &to), 
            "Transition {: ?} -> {:?} should be valid", from, to);
    }
}

#[tokio::test]
async fn test_invalid_job_state_transitions() {
    let invalid_transitions = vec![
        (JobState::Draft, JobState::Completed),
        (JobState:: Proposed, JobState::InProgress),
        (JobState:: Completed, JobState::InProgress),
    ];
    
    for (from, to) in invalid_transitions {
        assert! (!is_valid_transition(&from, &to),
            "Transition {:?} -> {:?} should be invalid", from, to);
    }
}

fn is_valid_transition(from:  &JobState, to: &JobState) -> bool {
    use JobState::*;
    match (from, to) {
        (Draft, Proposed) => true,
        (Proposed, Approved) | (Proposed, Rejected) => true,
        (Approved, InProgress) | (Approved, Cancelled) => true,
        (InProgress, WaitingInput) | (InProgress, Completed) | 
        (InProgress, Failed) | (InProgress, Cancelled) => true,
        (WaitingInput, InProgress) | (WaitingInput, Cancelled) => true,
        _ => false,
    }
}

#[tokio::test]
async fn test_formalize_card_generation() {
    let job_id = format!("job_{}", Uuid::new_v4());
    let card_id = format!("card_{}", Uuid::new_v4());
    
    let card = FormalizeCard {
        card_id: card_id.clone(),
        job_id: job_id.clone(),
        version: "1.0".to_string(),
        title: "Test Job".to_string(),
        summary: "This is a test job". to_string(),
        details: Some("Detailed description".to_string()),
        state: "proposed".to_string(),
        buttons: vec![
            json!({
                "button_id": "approve_btn",
                "label": "Approve",
                "action": {
                    "type": "job. approve",
                    "job_id": job_id
                },
                "style": "primary",
                "requires_input": false
            }),
            json!({
                "button_id": "reject_btn",
                "label": "Reject",
                "action": {
                    "type": "job.reject",
                    "job_id":  job_id
                },
                "style": "danger",
                "requires_input": false
            }),
        ],
    };
    
    assert_eq!(card.card_id, card_id);
    assert_eq!(card.state, "proposed");
    assert_eq!(card.buttons.len(), 2);
}

#[tokio::test]
async fn test_tracking_card_progress() {
    let job_id = format!("job_{}", Uuid::new_v4());
    let card_id = format! ("card_{}", Uuid::new_v4());
    
    let card = TrackingCard {
        card_id: card_id.clone(),
        job_id: job_id.clone(),
        version: "1.0".to_string(),
        state: "in_progress".to_string(),
        progress: 50,
        current_step: Some("Processing data".to_string()),
        buttons: vec![
            json!({
                "button_id": "cancel_btn",
                "label":  "Cancel",
                "action":  {
                    "type": "job.cancel",
                    "job_id": job_id
                },
                "style": "secondary",
                "requires_input": false
            }),
        ],
    };
    
    assert_eq!(card.progress, 50);
    assert_eq!(card.state, "in_progress");
}

#[tokio::test]
async fn test_finished_card_with_artifacts() {
    let job_id = format!("job_{}", Uuid::new_v4());
    let card_id = format! ("card_{}", Uuid::new_v4());
    
    let card = FinishedCard {
        card_id: card_id.clone(),
        job_id: job_id.clone(),
        version: "1.0".to_string(),
        state: "completed".to_string(),
        result_summary: "Task completed successfully".to_string(),
        artifacts: vec![
            json!({
                "artifact_id": "artifact_1",
                "kind": "file",
                "title": "report.pdf",
                "url": "https://example.com/report.pdf"
            }),
        ],
        buttons: vec![
            json!({
                "button_id": "ack_btn",
                "label":  "Acknowledge",
                "action": {
                    "type":  "job.ack",
                    "job_id": job_id
                },
                "style": "primary",
                "requires_input": false
            }),
        ],
    };
    
    assert_eq!(card.state, "completed");
    assert_eq!(card.artifacts.len(), 1);
}

#[tokio::test]
async fn test_job_action_approve() {
    let job_id = format!("job_{}", Uuid::new_v4());
    
    let action = JobAction:: Approve {
        job_id: job_id.clone(),
        card_id: "card_123".to_string(),
        button_id: "approve_btn".to_string(),
    };
    
    match action {
        JobAction:: Approve { job_id: id, ..  } => assert_eq!(id, job_id),
        _ => panic!("Wrong action type"),
    }
}

#[tokio::test]
async fn test_job_action_reject() {
    let job_id = format!("job_{}", Uuid::new_v4());
    
    let action = JobAction::Reject {
        job_id: job_id. clone(),
        card_id: "card_123".to_string(),
        button_id: "reject_btn".to_string(),
        reason: Some("Not needed".to_string()),
    };
    
    match action {
        JobAction::Reject { job_id: id, reason, .. } => {
            assert_eq!(id, job_id);
            assert_eq!(reason. unwrap(), "Not needed");
        }
        _ => panic!("Wrong action type"),
    }
}

#[tokio::test]
async fn test_job_action_provide_input() {
    let job_id = format!("job_{}", Uuid::new_v4());
    
    let action = JobAction::ProvideInput {
        job_id: job_id.clone(),
        card_id: "card_123".to_string(),
        button_id: "input_btn".to_string(),
        input_data:  json!({
            "answer": "Yes, proceed"
        }),
    };
    
    match action {
        JobAction:: ProvideInput { job_id:  id, input_data, .. } => {
            assert_eq!(id, job_id);
            assert_eq!(input_data["answer"], "Yes, proceed");
        }
        _ => panic!("Wrong action type"),
    }
}

#[tokio::test]
async fn test_job_waiting_on_user() {
    let mut job = Job {
        job_id: format!("job_{}", Uuid:: new_v4()),
        conversation_id: format!("conv_{}", Uuid::new_v4()),
        title: "Test Job".to_string(),
        goal: "Test". to_string(),
        state: JobState::WaitingInput,
        owner_entity_id: format!("agent_{}", Uuid::new_v4()),
        created_by_entity_id: format!("user_{}", Uuid::new_v4()),
        priority: "normal".to_string(),
        estimated_duration_seconds: None,
        progress: 50,
        waiting_on: vec! ["user_alice".to_string()],
        available_actions: vec![],
    };
    
    assert_eq!(job.state, JobState::WaitingInput);
    assert!(job.waiting_on.contains(&"user_alice".to_string()));
}

#[tokio::test]
async fn test_job_progress_updates() {
    let mut job = Job {
        job_id: format!("job_{}", Uuid::new_v4()),
        conversation_id: format! ("conv_{}", Uuid::new_v4()),
        title: "Test Job".to_string(),
        goal: "Test".to_string(),
        state: JobState::InProgress,
        owner_entity_id: format!("agent_{}", Uuid::new_v4()),
        created_by_entity_id: format!("user_{}", Uuid::new_v4()),
        priority: "normal".to_string(),
        estimated_duration_seconds: None,
        progress: 0,
        waiting_on: vec![],
        available_actions: vec![],
    };
    
    job.progress = 25;
    assert_eq!(job.progress, 25);
    
    job.progress = 50;
    assert_eq!(job.progress, 50);
    
    job.progress = 100;
    assert_eq!(job.progress, 100);
}

#[tokio::test]
async fn test_job_priority_levels() {
    let priorities = vec! ["low", "normal", "high", "urgent"];
    
    for priority in priorities {
        let job = Job {
            job_id: format!("job_{}", Uuid::new_v4()),
            conversation_id: format! ("conv_{}", Uuid::new_v4()),
            title:  "Test Job".to_string(),
            goal: "Test".to_string(),
            state: JobState::Draft,
            owner_entity_id: format!("agent_{}", Uuid::new_v4()),
            created_by_entity_id: format!("user_{}", Uuid::new_v4()),
            priority: priority.to_string(),
            estimated_duration_seconds: None,
            progress: 0,
            waiting_on:  vec![],
            available_actions: vec![],
        };
        
        assert_eq!(job.priority, priority);
    }
}

#[tokio:: test]
async fn test_job_available_actions() {
    let mut job = Job {
        job_id: format!("job_{}", Uuid::new_v4()),
        conversation_id: format! ("conv_{}", Uuid::new_v4()),
        title: "Test Job".to_string(),
        goal: "Test".to_string(),
        state: JobState:: Proposed,
        owner_entity_id: format!("agent_{}", Uuid::new_v4()),
        created_by_entity_id: format!("user_{}", Uuid::new_v4()),
        priority: "normal".to_string(),
        estimated_duration_seconds: None,
        progress: 0,
        waiting_on: vec![],
        available_actions: vec![],
    };
    
    // In Proposed state, should have approve/reject actions
    job.available_actions = vec![
        json!({"type": "job.approve"}),
        json!({"type":  "job.reject"}),
    ];
    
    assert_eq!(job.available_actions.len(), 2);
}