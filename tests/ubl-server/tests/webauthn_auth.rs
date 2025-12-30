//! WebAuthn Authentication Tests
//! Tests passkey registration, login, and step-up authentication

use sqlx::PgPool;
use uuid::Uuid;
use base64::{engine::general_purpose:: URL_SAFE_NO_PAD, Engine};

#[sqlx::test]
async fn test_user_registration_flow(pool: PgPool) {
    let username = "test_user";
    let display_name = "Test User";
    
    // 1. Check user doesn't exist
    let existing = sqlx::query!(
        r#"SELECT sid FROM id_subjects WHERE sid = $1"#,
        format!("user_{}", username)
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    
    assert!(existing.is_none());
    
    // 2. Create subject
    let sid = format!("user_{}", username);
    sqlx::query!(
        r#"
        INSERT INTO id_subjects (sid, kind, display_name, created_at_ms)
        VALUES ($1, 'person', $2, $3)
        "#,
        sid,
        display_name,
        1000000i64
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // 3. Store WebAuthn credential (mock)
    let credential_id = "mock_credential_id";
    let public_key = b"mock_public_key";
    
    sqlx::query!(
        r#"
        INSERT INTO id_credentials (
            credential_id, subject_sid, credential_kind, public_key, created_at_ms
        )
        VALUES ($1, $2, 'webauthn', $3, $4)
        "#,
        credential_id,
        sid,
        public_key,
        1000000i64
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // 4. Verify credential stored
    let cred = sqlx::query!(
        r#"
        SELECT credential_id, subject_sid, credential_kind
        FROM id_credentials
        WHERE credential_id = $1
        "#,
        credential_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(cred. credential_id, credential_id);
    assert_eq!(cred.subject_sid, sid);
    assert_eq!(cred. credential_kind, "webauthn");
}

#[sqlx::test]
async fn test_session_creation(pool: PgPool) {
    let sid = "user_test";
    let token = Uuid::new_v4().to_string();
    let exp_unix = 1000000i64 + 3600; // 1 hour from now
    
    // Create session
    sqlx::query!(
        r#"
        INSERT INTO id_session (token, sid, flavor, scope, exp_unix)
        VALUES ($1, $2, 'regular', $3, $4)
        "#,
        token,
        sid,
        serde_json::json!({}),
        exp_unix
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Retrieve session
    let session = sqlx::query!(
        r#"
        SELECT token, sid, flavor, scope, exp_unix
        FROM id_session
        WHERE token = $1
        "#,
        token
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(session.token, token);
    assert_eq!(session.sid, sid);
    assert_eq!(session.flavor, "regular");
}

#[sqlx::test]
async fn test_session_expiration(pool: PgPool) {
    let sid = "user_test";
    let expired_token = Uuid::new_v4().to_string();
    let valid_token = Uuid::new_v4().to_string();
    
    let now = 1000000i64;
    let expired_time = now - 3600; // 1 hour ago
    let valid_time = now + 3600; // 1 hour from now
    
    // Create expired session
    sqlx::query!(
        r#"
        INSERT INTO id_session (token, sid, flavor, scope, exp_unix)
        VALUES ($1, $2, 'regular', $3, $4)
        "#,
        expired_token,
        sid,
        serde_json::json!({}),
        expired_time
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Create valid session
    sqlx::query!(
        r#"
        INSERT INTO id_session (token, sid, flavor, scope, exp_unix)
        VALUES ($1, $2, 'regular', $3, $4)
        "#,
        valid_token,
        sid,
        serde_json::json!({}),
        valid_time
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Query only valid sessions
    let valid_sessions = sqlx::query!(
        r#"
        SELECT token
        FROM id_session
        WHERE exp_unix > $1
        "#,
        now
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(valid_sessions.len(), 1);
    assert_eq!(valid_sessions[0].token, valid_token);
}

#[sqlx::test]
async fn test_stepup_challenge_creation(pool: PgPool) {
    let challenge_id = Uuid::new_v4().to_string();
    let user_id = "user_test";
    let binding_hash = "blake3:abc123";
    let challenge_b64 = URL_SAFE_NO_PAD.encode(b"random_challenge");
    let auth_state = serde_json::json!({"state": "pending"});
    
    let created_at_ms = 1000000i64;
    let exp_ms = created_at_ms + 90000; // 90 seconds
    
    // Store challenge
    sqlx::query!(
        r#"
        INSERT INTO id_stepup_challenges (
            challenge_id, user_id, binding_hash, challenge_b64,
            auth_state, created_at_ms, exp_ms, used
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE)
        "#,
        challenge_id,
        user_id,
        binding_hash,
        challenge_b64,
        auth_state,
        created_at_ms,
        exp_ms
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Retrieve challenge
    let challenge = sqlx::query!(
        r#"
        SELECT challenge_id, user_id, binding_hash, used
        FROM id_stepup_challenges
        WHERE challenge_id = $1
        "#,
        challenge_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(challenge.challenge_id, challenge_id);
    assert_eq!(challenge.user_id, user_id);
    assert_eq!(challenge.binding_hash, binding_hash);
    assert_eq!(challenge.used, false);
}

#[sqlx::test]
async fn test_stepup_challenge_usage(pool: PgPool) {
    let challenge_id = Uuid::new_v4().to_string();
    let user_id = "user_test";
    let binding_hash = "blake3:abc123";
    
    // Create challenge
    sqlx::query!(
        r#"
        INSERT INTO id_stepup_challenges (
            challenge_id, user_id, binding_hash, challenge_b64,
            auth_state, created_at_ms, exp_ms, used
        )
        VALUES ($1, $2, $3, 'challenge', $4, 1000000, 1090000, FALSE)
        "#,
        challenge_id,
        user_id,
        binding_hash,
        serde_json::json! ({})
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Mark as used
    sqlx::query!(
        r#"
        UPDATE id_stepup_challenges
        SET used = TRUE
        WHERE challenge_id = $1
        "#,
        challenge_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Verify cannot be used again
    let challenge = sqlx::query!(
        r#"
        SELECT used
        FROM id_stepup_challenges
        WHERE challenge_id = $1
        "#,
        challenge_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(challenge.used, true);
}

#[sqlx:: test]
async fn test_asc_creation(pool: PgPool) {
    let sid = "agent_test";
    let containers = vec!["C.Jobs", "C.Messenger"];
    let intent_classes = vec!["Observation", "Conservation"];
    
    let scopes = serde_json::json!({
        "containers": containers,
        "intent_classes": intent_classes,
        "max_delta":  1000
    });
    
    let created_at_ms = 1000000i64;
    let not_before = created_at_ms;
    let not_after = created_at_ms + 86400000; // 24 hours
    
    // Create ASC
    sqlx::query!(
        r#"
        INSERT INTO id_asc (
            sid, scopes, not_before, not_after, signature, created_at_ms
        )
        VALUES ($1, $2, $3, $4, 'signature', $5)
        "#,
        sid,
        scopes,
        not_before,
        not_after,
        created_at_ms
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Retrieve ASC
    let asc = sqlx::query!(
        r#"
        SELECT sid, scopes, not_before, not_after
        FROM id_asc
        WHERE sid = $1
        "#,
        sid
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(asc.sid, sid);
    assert_eq!(asc.scopes["containers"], serde_json::json!(containers));
}

#[sqlx::test]
async fn test_asc_expiration_check(pool: PgPool) {
    let sid = "agent_test";
    let now = 1000000i64;
    
    // Create expired ASC
    sqlx::query!(
        r#"
        INSERT INTO id_asc (
            sid, scopes, not_before, not_after, signature, created_at_ms
        )
        VALUES ($1, $2, $3, $4, 'sig', $5)
        "#,
        sid,
        serde_json::json!({}),
        now - 10000,
        now - 1000, // Expired
        now - 10000
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Query valid ASCs (not_after > now)
    let valid_ascs = sqlx::query!(
        r#"
        SELECT sid
        FROM id_asc
        WHERE sid = $1 AND not_before <= $2 AND not_after > $2
        "#,
        sid,
        now
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(valid_ascs.len(), 0, "Expired ASC should not be returned");
}