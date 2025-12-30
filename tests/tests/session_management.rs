//! Session Management Tests
//! Tests session creation, lifecycle, token budgets, and handovers

use office::{
    session::{Session, SessionType, SessionMode, SessionStatus, Handover, HandoverBuilder},
    entity::Entity,
};
use uuid::Uuid;

#[tokio::test]
async fn test_session_creation() {
    let entity_id = format!("entity_{}", Uuid::new_v4());
    let session_id = format!("session_{}", Uuid:: new_v4());
    
    let session = Session::new(
        session_id.clone(),
        entity_id.clone(),
        SessionType::Work,
        SessionMode:: Commitment,
    );
    
    assert_eq!(session.id, session_id);
    assert_eq!(session.entity_id, entity_id);
    assert_eq!(session.session_type, SessionType::Work);
    assert_eq!(session. session_mode, SessionMode:: Commitment);
    assert_eq!(session.status, SessionStatus:: Pending);
}

#[tokio::test]
async fn test_session_types() {
    let types = vec![
        (SessionType::Work, 5000),
        (SessionType:: Assist, 4000),
        (SessionType:: Deliberate, 8000),
        (SessionType::Research, 6000),
    ];
    
    for (session_type, expected_budget) in types {
        let session = Session::new(
            format!("session_{}", Uuid:: new_v4()),
            format!("entity_{}", Uuid:: new_v4()),
            session_type. clone(),
            SessionMode:: Commitment,
        );
        
        assert_eq!(session.token_budget, expected_budget);
    }
}

#[tokio::test]
async fn test_session_modes() {
    let modes = vec![
        SessionMode::Commitment,
        SessionMode::Deliberation,
    ];
    
    for mode in modes {
        let session = Session::new(
            format!("session_{}", Uuid::new_v4()),
            format!("entity_{}", Uuid::new_v4()),
            SessionType:: Work,
            mode. clone(),
        );
        
        assert_eq!(session.session_mode, mode);
    }
}

#[tokio:: test]
async fn test_session_status_transitions() {
    let mut session = Session::new(
        format!("session_{}", Uuid:: new_v4()),
        format!("entity_{}", Uuid:: new_v4()),
        SessionType::Work,
        SessionMode::Commitment,
    );
    
    // Pending -> Active
    session.status = SessionStatus::Active;
    assert_eq!(session.status, SessionStatus::Active);
    
    // Active -> Paused
    session.status = SessionStatus::Paused;
    assert_eq!(session.status, SessionStatus::Paused);
    
    // Paused -> Active
    session.status = SessionStatus::Active;
    assert_eq!(session.status, SessionStatus:: Active);
    
    // Active -> Completed
    session.status = SessionStatus::Completed;
    assert_eq!(session.status, SessionStatus::Completed);
}

#[tokio::test]
async fn test_token_budget_consumption() {
    let mut session = Session::new(
        format! ("session_{}", Uuid::new_v4()),
        format!("entity_{}", Uuid::new_v4()),
        SessionType::Work,
        SessionMode:: Commitment,
    );
    
    assert_eq!(session.tokens_consumed, 0);
    assert_eq!(session.token_budget, 5000);
    
    // Consume tokens
    session.consume_tokens(1000);
    assert_eq!(session.tokens_consumed, 1000);
    assert_eq!(session.remaining_budget(), 4000);
    
    session.consume_tokens(2000);
    assert_eq!(session. tokens_consumed, 3000);
    assert_eq!(session. remaining_budget(), 2000);
}

#[tokio:: test]
async fn test_token_budget_exceeded() {
    let mut session = Session::new(
        format! ("session_{}", Uuid::new_v4()),
        format!("entity_{}", Uuid::new_v4()),
        SessionType::Work,
        SessionMode:: Commitment,
    );
    
    session.consume_tokens(5001);
    
    assert!(session.tokens_consumed > session.token_budget);
    assert! (!session.within_budget());
}

#[tokio::test]
async fn test_handover_creation() {
    let entity_id = format!("entity_{}", Uuid:: new_v4());
    let session_id = format!("session_{}", Uuid::new_v4());
    let instance_id = format!("instance_{}", Uuid::new_v4());
    
    let handover = HandoverBuilder::new(entity_id.clone(), session_id.clone(), instance_id.clone())
        .accomplished(vec! ["Fixed bug #123".to_string(), "Updated docs".to_string()])
        .open_threads(vec![
            ("Need to add tests". to_string(), 5),
        ])
        .observations(vec! ["Code quality is good".to_string()])
        .emotional_note("Feeling confident about progress")
        .build();
    
    assert_eq!(handover.entity_id, entity_id);
    assert_eq!(handover.session_id, session_id);
    assert_eq!(handover.instance_id, instance_id);
    assert_eq!(handover.accomplished. len(), 2);
    assert_eq!(handover.open_threads.len(), 1);
}

#[tokio:: test]
async fn test_handover_summary_generation() {
    let handover = HandoverBuilder::new(
        format!("entity_{}", Uuid::new_v4()),
        format!("session_{}", Uuid::new_v4()),
        format!("instance_{}", Uuid::new_v4()),
    )
    .accomplished(vec!["Task 1".to_string()])
    .build();
    
    let summary = handover.generate_summary();
    
    assert!(summary.contains("Accomplished"));
    assert!(summary.contains("Task 1"));
}

#[tokio::test]
async fn test_handover_keyword_extraction() {
    let handover = HandoverBuilder::new(
        format!("entity_{}", Uuid::new_v4()),
        format!("session_{}", Uuid::new_v4()),
        format!("instance_{}", Uuid::new_v4()),
    )
    .observations(vec![
        "System is working well".to_string(),
        "Found a critical issue".to_string(),
        "Urgent attention needed".to_string(),
    ])
    .build();
    
    let keywords = handover.extract_keywords();
    
    assert!(keywords.contains(&"critical".to_string()) || keywords.contains(&"urgent".to_string()));
}

#[tokio::test]
async fn test_emotional_state() {
    let handover = HandoverBuilder::new(
        format!("entity_{}", Uuid:: new_v4()),
        format!("session_{}", Uuid:: new_v4()),
        format!("instance_{}", Uuid:: new_v4()),
    )
    .emotional_note("Feeling satisfied with the work completed")
    .build();
    
    assert!(handover.emotional_state. is_some());
    let state = handover.emotional_state.unwrap();
    assert!(state.satisfaction > 0.5);
}

#[tokio::test]
async fn test_session_instance_count() {
    let mut session = Session::new(
        format! ("session_{}", Uuid::new_v4()),
        format!("entity_{}", Uuid::new_v4()),
        SessionType::Work,
        SessionMode:: Commitment,
    );
    
    assert_eq!(session. instance_count, 0);
    
    session.instance_count += 1;
    assert_eq!(session.instance_count, 1);
}

#[tokio::test]
async fn test_session_message_count() {
    let mut session = Session::new(
        format! ("session_{}", Uuid::new_v4()),
        format!("entity_{}", Uuid::new_v4()),
        SessionType::Work,
        SessionMode:: Commitment,
    );
    
    assert_eq!(session. message_count, 0);
    
    session.message_count += 1;
    assert_eq!(session.message_count, 1);
}