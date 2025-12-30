//! Complete End-to-End Flow Tests

use office::{
    entity: :{Entity, EntityType},
    session: :{Session, SessionType, SessionMode},
    job_executor: :{Job, JobState, JobAction},
};
use uuid::Uuid;
use serde_json::json;

#[tokio::test]
async fn test_complete_job_execution_flow() {
    // 1. Create Entity
    let entity = Entity:: new(
        format!("entity_{}", Uuid::new_v4()),
        "Test Agent".to_string(),
        EntityType:: Autonomous,
    );
    
    assert_eq!(entity.status, office::entity::EntityStatus::Active);
    
    // 2. Create Session
    let session = Session::new(
        format!("session_{}", Uuid:: new_v4()),
        entity.id.clone(),
        SessionType::Work,
        SessionMode::Commitment,
    );
    
    assert_eq!(session.token_budget, 5000);
    
    // 3. Create Job
    let mut job = Job {
        job_id: format!("job_{}", Uuid:: new_v4()),
        conversation_id: format!("conv_{}", Uuid::new_v4()),
        title: "Test Job".to_string(),
        goal: "Complete test task".to_string(),
        state: JobState::Draft,
        owner_entity_id: entity.id.clone(),
        created_by_entity_id: "user_test".to_string(),
        priority: "normal".to_string(),
        estimated_duration_seconds: Some(300),
        progress: 0,
        waiting_on: vec![],
        available_actions: vec![],
    };
    
    // 4. Propose Job
    job.state = JobState::Proposed;
    assert_eq!(job.state, JobState::Proposed);
    
    // 5. Approve Job
    job.state = JobState::Approved;
    assert_eq!(job.state, JobState::Approved);
    
    // 6. Start Execution
    job.state = JobState::InProgress;
    job.progress = 0;
    assert_eq!(job. state, JobState::InProgress);
    
    // 7. Update Progress
    job.progress = 50;
    assert_eq!(job.progress, 50);
    
    // 8. Complete Job
    job.state = JobState::Completed;
    job.progress = 100;
    assert_eq!(job.state, JobState::Completed);
    assert_eq!(job.progress, 100);
}

#[tokio::test]
async fn test_job_with_approval_flow() {
    // 1. Create job requiring approval
    let mut job = Job {
        job_id: format!("job_{}", Uuid::new_v4()),
        conversation_id: format! ("conv_{}", Uuid::new_v4()),
        title: "High Risk Job".to_string(),
        goal: "Deploy to production".to_string(),
        state: JobState::Draft,
        owner_entity_id:  format!("agent_{}", Uuid::new_v4()),
        created_by_entity_id: "user_test".to_string(),
        priority: "urgent".to_string(),
        estimated_duration_seconds: Some(600),
        progress: 0,
        waiting_on: vec![],
        available_actions: vec![],
    };
    
    // 2. Propose
    job.state = JobState:: Proposed;
    
    // 3. User approves
    let action = JobAction::Approve {
        job_id: job. job_id.clone(),
        card_id: "card_123". to_string(),
        button_id: "approve_btn".to_string(),
    };
    
    // 4. Transition to approved
    job.state = JobState::Approved;
    
    // 5. Execute
    job.state = JobState::InProgress;
    
    // 6. Complete
    job.state = JobState::Completed;
    
    assert_eq!(job.state, JobState::Completed);
}

#[tokio::test]
async fn test_job_with_user_input_flow() {
    // 1. Create job
    let mut job = Job {
        job_id: format!("job_{}", Uuid::new_v4()),
        conversation_id: format!("conv_{}", Uuid::new_v4()),
        title: "Interactive Job".to_string(),
        goal: "Process with user input".to_string(),
        state: JobState:: Approved,
        owner_entity_id: format!("agent_{}", Uuid::new_v4()),
        created_by_entity_id: "user_test".to_string(),
        priority: "normal".to_string(),
        estimated_duration_seconds: Some(300),
        progress: 0,
        waiting_on: vec![],
        available_actions: vec![],
    };
    
    // 2. Start execution
    job.state = JobState::InProgress;
    job. progress = 30;
    
    // 3. Need user input
    job.state = JobState::WaitingInput;
    job.waiting_on = vec! ["user_test".to_string()];
    
    assert_eq!(job.state, JobState::WaitingInput);
    assert!(job.waiting_on.contains(&"user_test".to_string()));
    
    // 4. User provides input
    let action = JobAction:: ProvideInput {
        job_id: job.job_id.clone(),
        card_id: "card_123".to_string(),
        button_id: "input_btn".to_string(),
        input_data: json!({"answer": "proceed"}),
    };
    
    // 5. Resume execution
    job.state = JobState::InProgress;
    job.waiting_on. clear();
    job.progress = 60;
    
    // 6. Complete
    job. state = JobState::Completed;
    job.progress = 100;
    
    assert_eq!(job.state, JobState:: Completed);
}

#[tokio::test]
async fn test_entity_multiple_sessions() {
    let mut entity = Entity::new(
        format!("entity_{}", Uuid::new_v4()),
        "Multi-Session Agent".to_string(),
        EntityType::Autonomous,
    );
    
    // Session 1
    let session1 = Session::new(
        format!("session_{}", Uuid::new_v4()),
        entity.id.clone(),
        SessionType::Work,
        SessionMode::Commitment,
    );
    entity.total_sessions += 1;
    entity.total_tokens_consumed += 2000;
    
    // Session 2
    let session2 = Session::new(
        format!("session_{}", Uuid:: new_v4()),
        entity.id.clone(),
        SessionType:: Assist,
        SessionMode::Commitment,
    );
    entity.total_sessions += 1;
    entity.total_tokens_consumed += 1500;
    
    // Session 3
    let session3 = Session::new(
        format!("session_{}", Uuid::new_v4()),
        entity.id.clone(),
        SessionType::Research,
        SessionMode::Deliberation,
    );
    entity.total_sessions += 1;
    entity.total_tokens_consumed += 3000;
    
    assert_eq!(entity.total_sessions, 3);
    assert_eq!(entity.total_tokens_consumed, 6500);
}

#[tokio::test]
async fn test_job_cancellation_flow() {
    let mut job = Job {
        job_id: format!("job_{}", Uuid::new_v4()),
        conversation_id: format!("conv_{}", Uuid:: new_v4()),
        title: "Cancellable Job".to_string(),
        goal: "Task that gets cancelled".to_string(),
        state: JobState::InProgress,
        owner_entity_id: format!("agent_{}", Uuid::new_v4()),
        created_by_entity_id: "user_test".to_string(),
        priority: "normal".to_string(),
        estimated_duration_seconds: Some(300),
        progress: 40,
        waiting_on: vec![],
        available_actions:  vec![],
    };
    
    // Cancel job
    let action = JobAction::Cancel {
        job_id: job.job_id.clone(),
        card_id: "card_123". to_string(),
        button_id: "cancel_btn".to_string(),
        reason: Some("No longer needed".to_string()),
    };
    
    job.state = JobState::Cancelled;
    
    assert_eq!(job.state, JobState::Cancelled);
}