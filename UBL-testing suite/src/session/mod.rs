// Existing session code...

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_new() {
        let session = Session::new(
            "session_123".to_string(),
            "entity_456".to_string(),
            SessionType::Work,
            SessionMode::Commitment,
        );
        
        assert_eq!(session.id, "session_123");
        assert_eq!(session.entity_id, "entity_456");
        assert_eq!(session.session_type, SessionType::Work);
        assert_eq!(session.token_budget, 5000);
    }
    
    #[test]
    fn test_token_consumption() {
        let mut session = Session::new(
            "s1".to_string(),
            "e1".to_string(),
            SessionType::Work,
            SessionMode::Commitment,
        );
        
        session. consume_tokens(1000);
        assert_eq!(session.tokens_consumed, 1000);
        assert_eq!(session.remaining_budget(), 4000);
        
        session.consume_tokens(2000);
        assert_eq!(session.tokens_consumed, 3000);
        assert_eq!(session.remaining_budget(), 2000);
    }
    
    #[test]
    fn test_within_budget() {
        let mut session = Session::new(
            "s1".to_string(),
            "e1".to_string(),
            SessionType::Work,
            SessionMode::Commitment,
        );
        
        session.consume_tokens(4999);
        assert!(session.within_budget());
        
        session.consume_tokens(2);
        assert!(!session.within_budget());
    }
    
    #[test]
    fn test_session_types_budget() {
        let work = Session::new("s1".to_string(), "e1".to_string(), SessionType::Work, SessionMode:: Commitment);
        let assist = Session::new("s2". to_string(), "e1".to_string(), SessionType::Assist, SessionMode::Commitment);
        let deliberate = Session::new("s3".to_string(), "e1".to_string(), SessionType::Deliberate, SessionMode:: Commitment);
        let research = Session::new("s4". to_string(), "e1".to_string(), SessionType::Research, SessionMode::Commitment);
        
        assert_eq!(work.token_budget, 5000);
        assert_eq!(assist.token_budget, 4000);
        assert_eq!(deliberate.token_budget, 8000);
        assert_eq!(research.token_budget, 6000);
    }
}