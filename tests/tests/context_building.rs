//! Context Building Tests
//! Tests context frame construction, memory management, and narrative generation

use office::{
    context: :{ContextFrame, ContextBuilder, Memory, MemoryEntry, Narrator},
    entity::Entity,
    session: :{Session, SessionType, SessionMode},
    governance::constitution::OfficeConstitution,
};
use uuid:: Uuid;
use time::OffsetDateTime;

#[tokio::test]
async fn test_context_frame_creation() {
    let entity_id = format!("entity_{}", Uuid::new_v4());
    let entity_name = "Test Entity";
    
    let frame = ContextFrame {
        entity_id: entity_id.clone(),
        entity_name: entity_name.to_string(),
        session_type: SessionType::Work,
        token_budget: 5000,
        memory:  Memory::new(),
        obligations: vec![],
        affordances: vec![],
        constitution: OfficeConstitution:: default(),
        previous_handover: None,
        governance_notes: vec![],
        guardian_info: None,
        ledger_sequence: 100,
        frame_hash: "hash123".to_string(),
    };
    
    assert_eq!(frame.entity_id, entity_id);
    assert_eq!(frame.entity_name, entity_name);
    assert_eq!(frame.ledger_sequence, 100);
}

#[tokio::test]
async fn test_memory_creation() {
    let memory = Memory::new();
    
    assert_eq!(memory.recent_events.len(), 0);
    assert_eq!(memory.historical_syntheses.len(), 0);
    assert_eq!(memory.bookmarks.len(), 0);
    assert_eq!(memory.total_events, 0);
}

#[tokio::test]
async fn test_memory_add_event() {
    let mut memory = Memory::new();
    
    let entry = MemoryEntry {
        event_id: "event_1".to_string(),
        event_type: "job. created".to_string(),
        timestamp: OffsetDateTime::now_utc(),
        summary: "Created new job".to_string(),
        data: Some(serde_json::json!({"job_id": "job_123"})),
        is_bookmarked: false,
    };
    
    memory.add_event(entry. clone());
    
    assert_eq!(memory.recent_events. len(), 1);
    assert_eq!(memory.total_events, 1);
    assert_eq!(memory.recent_events[0]. event_id, "event_1");
}

#[tokio::test]
async fn test_memory_bookmarking() {
    let mut memory = Memory::new();
    
    let mut entry = MemoryEntry {
        event_id: "event_1".to_string(),
        event_type: "job.completed".to_string(),
        timestamp: OffsetDateTime::now_utc(),
        summary: "Important milestone". to_string(),
        data: None,
        is_bookmarked: true,
    };
    
    memory.add_event(entry. clone());
    memory.add_bookmark("event_1", "Critical milestone achieved");
    
    assert_eq!(memory.bookmarks.len(), 1);
}

#[tokio::test]
async fn test_memory_token_estimation() {
    let mut memory = Memory::new();
    
    // Add some events
    for i in 0..10 {
        let entry = MemoryEntry {
            event_id:  format!("event_{}", i),
            event_type: "message.created".to_string(),
            timestamp: OffsetDateTime::now_utc(),
            summary: format!("Event {} summary", i),
            data: None,
            is_bookmarked: false,
        };
        memory.add_event(entry);
    }
    
    let token_estimate = memory.estimate_tokens();
    
    // Should be > 0
    assert!(token_estimate > 0);
}

#[tokio::test]
async fn test_memory_compression() {
    let mut memory = Memory::new();
    
    // Add many events
    for i in 0..30 {
        let entry = MemoryEntry {
            event_id: format!("event_{}", i),
            event_type:  "message.created".to_string(),
            timestamp: OffsetDateTime::now_utc(),
            summary: format!("Event {} summary with some long text to increase size", i),
            data: Some(serde_json::json!({"large":  "data structure here"})),
            is_bookmarked: false,
        };
        memory.add_event(entry);
    }
    
    let before_compression = memory.estimate_tokens();
    
    // Compress memory
    memory.compress(5000); // Target 5000 tokens
    
    let after_compression = memory. estimate_tokens();
    
    // Should be smaller or equal
    assert!(after_compression <= before_compression);
    assert!(after_compression <= 5000);
}

#[tokio::test]
async fn test_narrator_identity_section() {
    let frame = ContextFrame {
        entity_id: format!("entity_{}", Uuid::new_v4()),
        entity_name: "Sofia".to_string(),
        session_type: SessionType::Work,
        token_budget: 5000,
        memory: Memory:: new(),
        obligations: vec![],
        affordances: vec![],
        constitution: OfficeConstitution::default(),
        previous_handover: None,
        governance_notes: vec![],
        guardian_info: None,
        ledger_sequence: 100,
        frame_hash:  "hash123".to_string(),
    };
    
    let narrator = Narrator::new();
    let narrative = narrator.generate_narrative(&frame);
    
    assert!(narrative.contains("Sofia"));
    assert!(narrative.contains("IDENTITY"));
}

#[tokio::test]
async fn test_narrator_situation_section() {
    let frame = ContextFrame {
        entity_id: format!("entity_{}", Uuid::new_v4()),
        entity_name: "Sofia".to_string(),
        session_type: SessionType::Work,
        token_budget: 5000,
        memory: Memory::new(),
        obligations: vec![],
        affordances: vec![],
        constitution: OfficeConstitution::default(),
        previous_handover: None,
        governance_notes: vec![],
        guardian_info: None,
        ledger_sequence:  100,
        frame_hash: "hash123".to_string(),
    };
    
    let narrator = Narrator::new();
    let narrative = narrator.generate_narrative(&frame);
    
    assert!(narrative.contains("CURRENT SITUATION"));
    assert!(narrative.contains("autonomous work session"));
    assert!(narrative.contains("5000 tokens"));
}

#[tokio::test]
async fn test_narrator_constitution_last() {
    let frame = ContextFrame {
        entity_id:  format!("entity_{}", Uuid::new_v4()),
        entity_name: "Sofia".to_string(),
        session_type: SessionType::Work,
        token_budget: 5000,
        memory: Memory::new(),
        obligations: vec![],
        affordances: vec![],
        constitution: OfficeConstitution::default(),
        previous_handover:  None,
        governance_notes:  vec![],
        guardian_info: None,
        ledger_sequence: 100,
        frame_hash: "hash123".to_string(),
    };
    
    let narrator = Narrator:: new();
    let narrative = narrator.generate_narrative(&frame);
    
    // Constitution should appear last
    let constitution_pos = narrative.find("CONSTITUTION").unwrap();
    
    // Should be near the end
    assert!(constitution_pos > narrative.len() / 2);
}

#[tokio::test]
async fn test_context_builder() {
    let entity_id = format!("entity_{}", Uuid::new_v4());
    
    let builder = ContextBuilder:: new(entity_id.clone(), "Test Entity". to_string());
    
    let frame = builder
        .session_type(SessionType::Work)
        .token_budget(5000)
        .add_recent_event("event_1", "message.created", "User sent message")
        .build();
    
    assert_eq!(frame.entity_id, entity_id);
    assert_eq!(frame.session_type, SessionType::Work);
    assert_eq!(frame. memory.recent_events.len(), 1);
}