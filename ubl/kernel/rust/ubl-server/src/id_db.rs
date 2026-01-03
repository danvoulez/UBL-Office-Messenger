//! # UBL ID Database Layer
//!
//! Database operations for identity management:
//! - Subjects (person, llm, app)
//! - Credentials (passkey, ed25519)
//! - Challenges (WebAuthn)
//! - Sessions (user, ICT)
//! - ASC (Agent Signing Certificates)

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubjectKind {
    Person,
    Llm,
    App,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub sid: String,
    pub kind: String,
    pub display_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: Uuid,
    pub sid: String,
    pub credential_kind: String,
    pub credential_id: Option<String>,
    pub public_key: Vec<u8>,
    pub sign_count: i64,
    pub key_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Challenge {
    pub id: Uuid,
    pub kind: String,
    pub sid: Option<String>,
    pub challenge: Vec<u8>,
    pub origin: String,
    pub expires_at: OffsetDateTime,
    pub used: bool,
}

/// Alias for sqlx::FromRow compatibility
type ChallengeRow = Challenge;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub sid: String,
    pub session_id: Uuid,
    pub flavor: String,
    pub scope: serde_json::Value,
    pub not_before: OffsetDateTime,
    pub not_after: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asc {
    pub asc_id: Uuid,
    pub sid: String,
    pub public_key: Vec<u8>,
    pub scopes: serde_json::Value,
    pub not_before: OffsetDateTime,
    pub not_after: OffsetDateTime,
    pub signature: Vec<u8>,
}

// ============================================================================
// SUBJECT OPERATIONS
// ============================================================================

/// Create agent (LLM or App) with Ed25519 public key
/// sid = "ubl:sid:" + blake3(pubkey_hex | kind)
pub async fn create_agent(
    pool: &PgPool,
    kind: &str,
    display_name: &str,
    public_key_hex: &str,
) -> sqlx::Result<Subject> {
    // Compute stable SID
    let mut h = Hasher::new();
    h.update(public_key_hex.as_bytes());
    h.update(kind.as_bytes());
    let sid = format!("ubl:sid:{}", hex::encode(h.finalize().as_bytes()));

    // Insert subject
    sqlx::query!(
        r#"
        INSERT INTO id_subject (sid, kind, display_name)
        VALUES ($1, $2, $3)
        ON CONFLICT (sid) DO NOTHING
        "#,
        sid,
        kind,
        display_name
    )
    .execute(pool)
    .await?;

    // Insert Ed25519 credential
    let public_key_bytes = hex::decode(public_key_hex).map_err(|_| {
        sqlx::Error::Decode("Invalid hex public key".into())
    })?;

    sqlx::query!(
        r#"
        INSERT INTO id_credential (sid, credential_kind, public_key, key_version)
        VALUES ($1, 'ed25519', $2, 1)
        ON CONFLICT (sid, credential_kind, key_version) DO NOTHING
        "#,
        sid,
        public_key_bytes
    )
    .execute(pool)
    .await?;

    Ok(Subject {
        sid: sid.clone(),
        kind: kind.to_string(),
        display_name: display_name.to_string(),
        status: "active".to_string(),
    })
}

/// Get subject by SID
pub async fn get_subject(pool: &PgPool, sid: &str) -> sqlx::Result<Option<Subject>> {
    let row = sqlx::query!(
        r#"
        SELECT sid, kind, display_name, status
        FROM id_subject
        WHERE sid = $1
        "#,
        sid
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Subject {
        sid: r.sid,
        kind: r.kind,
        display_name: r.display_name,
        status: r.status,
    }))
}

/// Compute the stable SID for a person from their username
/// This MUST match the SID used in create_person()
pub fn compute_person_sid(username: &str) -> String {
    let mut h = Hasher::new();
    h.update(username.as_bytes());
    h.update(b"person");
    format!("ubl:sid:{}", hex::encode(h.finalize().as_bytes()))
}

/// Create person subject (WebAuthn)
pub async fn create_person(
    pool: &PgPool,
    username: &str,
    display_name: &str,
) -> sqlx::Result<String> {
    // Use the same SID computation
    let sid = compute_person_sid(username);

    sqlx::query!(
        r#"
        INSERT INTO id_subject (sid, kind, display_name)
        VALUES ($1, 'person', $2)
        ON CONFLICT (sid) DO NOTHING
        "#,
        sid,
        display_name
    )
    .execute(pool)
    .await?;

    Ok(sid)
}

/// Get subject by username (for people)
pub async fn get_subject_by_username(pool: &PgPool, username: &str) -> sqlx::Result<Option<Subject>> {
    // Recompute SID from username
    let mut h = Hasher::new();
    h.update(username.as_bytes());
    h.update(b"person");
    let sid = format!("ubl:sid:{}", hex::encode(h.finalize().as_bytes()));

    get_subject(pool, &sid).await
}

/// Get all credentials for a subject
pub async fn get_credentials(pool: &PgPool, sid: &str) -> sqlx::Result<Vec<Credential>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, sid, credential_kind as kind, credential_id, public_key, sign_count, key_version
        FROM id_credential
        WHERE sid = $1 AND NOT EXISTS (
            SELECT 1 FROM id_key_revocation
            WHERE id_key_revocation.sid = id_credential.sid
            AND id_key_revocation.key_version = id_credential.key_version
        )
        ORDER BY key_version DESC
        "#,
        sid
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| Credential {
        id: r.id,
        sid: r.sid,
        credential_kind: r.kind,
        credential_id: r.credential_id,
        public_key: r.public_key,
        sign_count: r.sign_count.unwrap_or(0),
        key_version: r.key_version,
    }).collect())
}

/// Create credential for subject
pub async fn create_credential(
    pool: &PgPool,
    sid: &str,
    kind: &str,
    credential_id: &str,
    public_key: &[u8],
    sign_count: i64,
) -> sqlx::Result<Uuid> {
    let row = sqlx::query!(
        r#"
        INSERT INTO id_credential (sid, credential_kind, credential_id, public_key, sign_count, key_version)
        VALUES ($1, $2, $3, $4, $5, 1)
        RETURNING id
        "#,
        sid,
        kind,
        credential_id,
        public_key,
        sign_count
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Get credential by credential_id
pub async fn get_credential_by_id(
    pool: &PgPool,
    sid: &str,
    credential_id: &str,
) -> sqlx::Result<Option<Credential>> {
    let row = sqlx::query!(
        r#"
        SELECT id, sid, credential_kind as kind, credential_id, public_key, sign_count, key_version
        FROM id_credential
        WHERE sid = $1 AND credential_id = $2
        "#,
        sid,
        credential_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Credential {
        id: r.id,
        sid: r.sid,
        credential_kind: r.kind,
        credential_id: r.credential_id,
        public_key: r.public_key,
        sign_count: r.sign_count.unwrap_or(0),
        key_version: r.key_version,
    }))
}

/// Get credential by credential_id only (for discoverable login)
pub async fn get_credential_by_cred_id_only(
    pool: &PgPool,
    credential_id: &str,
) -> sqlx::Result<Option<Credential>> {
    let row = sqlx::query_as::<_, (Uuid, String, String, Option<String>, Vec<u8>, Option<i64>, i32)>(
        r#"
        SELECT id, sid, credential_kind, credential_id, public_key, sign_count, key_version
        FROM id_credential
        WHERE credential_id = $1
        "#
    )
    .bind(credential_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, sid, kind, credential_id, public_key, sign_count, key_version)| Credential {
        id,
        sid,
        credential_kind: kind,
        credential_id,
        public_key,
        sign_count: sign_count.unwrap_or(0),
        key_version,
    }))
}

/// Create session
pub async fn create_session(
    pool: &PgPool,
    sid: &str,
    token: &str,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + time::Duration::seconds(ttl_secs);
    let exp_unix = not_after.unix_timestamp();
    let session_uuid = Uuid::new_v4();

    let row = sqlx::query!(
        r#"
        INSERT INTO id_session (token, sid, session_id, flavor, scope, exp_unix, not_before, not_after)
        VALUES ($1, $2, $3, 'webauthn', '{}', $4, $5, $6)
        RETURNING session_id
        "#,
        token,
        sid,
        session_uuid,
        exp_unix,
        not_before,
        not_after
    )
    .fetch_one(pool)
    .await?;

    Ok(row.session_id.unwrap_or(session_uuid))
}

/// Get challenge by ID
pub async fn get_challenge(pool: &PgPool, challenge_id: &str) -> sqlx::Result<Option<Challenge>> {
    let id = Uuid::parse_str(challenge_id)
        .map_err(|_| sqlx::Error::Decode("Invalid UUID".into()))?;

    let row = sqlx::query!(
        r#"
        SELECT id, kind, sid, challenge, origin, expires_at, used
        FROM id_challenge
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Challenge {
        id: r.id,
        kind: r.kind,
        sid: r.sid,
        challenge: r.challenge,
        origin: r.origin,
        expires_at: r.expires_at,
        used: r.used,
    }))
}

// ============================================================================
// CHALLENGE OPERATIONS
// ============================================================================

/// Create WebAuthn challenge for registration
pub async fn create_register_challenge(
    pool: &PgPool,
    username: &str,
    challenge_bytes: Vec<u8>,
    origin: &str,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(ttl_secs);

    // Store username + state in challenge as JSON
    let challenge_data = serde_json::json!({
        "username": username,
        "state": challenge_bytes
    });
    let challenge_json = serde_json::to_vec(&challenge_data)
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to encode challenge: {}", e)))?;

    let row = sqlx::query!(
        r#"
        INSERT INTO id_challenge (kind, challenge, origin, expires_at)
        VALUES ('register', $1, $2, $3)
        RETURNING id
        "#,
        challenge_json,
        origin,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Create WebAuthn challenge for login
pub async fn create_login_challenge(
    pool: &PgPool,
    sid: &str,
    challenge_bytes: Vec<u8>,
    origin: &str,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(ttl_secs);

    let row = sqlx::query!(
        r#"
        INSERT INTO id_challenge (kind, sid, challenge, origin, expires_at)
        VALUES ('login', $1, $2, $3, $4)
        RETURNING id
        "#,
        sid,
        challenge_bytes,
        origin,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Create WebAuthn challenge for discoverable login (no SID known yet)
pub async fn create_discoverable_challenge(
    pool: &PgPool,
    challenge_bytes: Vec<u8>,
    origin: &str,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(ttl_secs);
    let null_sid: Option<&str> = None;

    let row = sqlx::query_as::<_, (Uuid,)>(
        r#"
        INSERT INTO id_challenge (kind, sid, challenge, origin, expires_at)
        VALUES ('login', $1, $2, $3, $4)
        RETURNING id
        "#
    )
    .bind(null_sid)
    .bind(challenge_bytes)
    .bind(origin)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

/// Create WebAuthn challenge for step-up
pub async fn create_stepup_challenge(
    pool: &PgPool,
    sid: &str,
    challenge_bytes: Vec<u8>,
    origin: &str,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(ttl_secs);

    let row = sqlx::query!(
        r#"
        INSERT INTO id_challenge (kind, sid, challenge, origin, expires_at)
        VALUES ('stepup', $1, $2, $3, $4)
        RETURNING id
        "#,
        sid,
        challenge_bytes,
        origin,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

/// Atomically consume a challenge - prevents replay attacks (Diamond Checklist #4)
/// 
/// Uses UPDATE ... WHERE used = false RETURNING to ensure:
/// 1. Challenge exists
/// 2. Challenge not already used  
/// 3. Atomic consumption (no race condition window)
///
/// If the challenge was already used or doesn't exist, returns Ok(None).
pub async fn consume_challenge(
    pool: &PgPool,
    challenge_id: Uuid,
    expected_origin: &str,
) -> sqlx::Result<Option<Challenge>> {
    // Atomic UPDATE: only succeeds if used=false, sets used=true in same statement
    // This eliminates the race condition window between check and update
    // Uses raw query to avoid SQLX_OFFLINE cache issues
    let row: Option<ChallengeRow> = sqlx::query_as(
        r#"
        UPDATE id_challenge
        SET used = true
        WHERE id = $1 
          AND used = false 
          AND origin = $2
          AND expires_at > NOW()
        RETURNING id, kind, sid, challenge, origin, expires_at, used
        "#
    )
    .bind(challenge_id)
    .bind(expected_origin)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Challenge {
        id: r.id,
        kind: r.kind,
        sid: r.sid,
        challenge: r.challenge,
        origin: r.origin,
        expires_at: r.expires_at,
        used: r.used,
    }))
}

// ============================================================================
// SESSION OPERATIONS
// ============================================================================

/// Create user session
pub async fn create_user_session(
    pool: &PgPool,
    sid: &str,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + time::Duration::seconds(ttl_secs);
    let exp_unix = not_after.unix_timestamp();
    let token = Uuid::new_v4().to_string();
    let session_uuid = Uuid::new_v4();

    let row = sqlx::query!(
        r#"
        INSERT INTO id_session (token, sid, session_id, flavor, scope, exp_unix, not_before, not_after)
        VALUES ($1, $2, $3, 'user', '{}'::jsonb, $4, $5, $6)
        RETURNING session_id
        "#,
        token,
        sid,
        session_uuid,
        exp_unix,
        not_before,
        not_after
    )
    .fetch_one(pool)
    .await?;

    Ok(row.session_id.unwrap_or(session_uuid))
}

/// Create step-up session (admin, short TTL)
pub async fn create_stepup_session(
    pool: &PgPool,
    sid: &str,
    token: &str,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + time::Duration::seconds(ttl_secs);
    let exp_unix = not_after.unix_timestamp();
    let session_uuid = Uuid::new_v4();

    let row = sqlx::query!(
        r#"
        INSERT INTO id_session (token, sid, session_id, flavor, scope, exp_unix, not_before, not_after)
        VALUES ($1, $2, $3, 'stepup', '{"role":"admin"}', $4, $5, $6)
        RETURNING session_id
        "#,
        token,
        sid,
        session_uuid,
        exp_unix,
        not_before,
        not_after
    )
    .fetch_one(pool)
    .await?;

    Ok(row.session_id.unwrap_or(session_uuid))
}

/// Get active session by session_id UUID
pub async fn get_session(pool: &PgPool, session_id: Uuid) -> sqlx::Result<Option<Session>> {
    let row = sqlx::query!(
        r#"
        SELECT sid, session_id, flavor, scope, not_before, not_after
        FROM id_session
        WHERE session_id = $1
        "#,
        session_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|r| {
        let now = OffsetDateTime::now_utc();
        let not_before = r.not_before?;
        let not_after = r.not_after?;
        let sess_id = r.session_id?;
        if now >= not_before && now <= not_after {
            Some(Session {
                sid: r.sid,
                session_id: sess_id,
                flavor: r.flavor,
                scope: r.scope,
                not_before,
                not_after,
            })
        } else {
            None // Expired
        }
    }))
}

// ============================================================================
// ASC OPERATIONS
// ============================================================================

/// Issue Agent Signing Certificate
pub async fn issue_asc(
    pool: &PgPool,
    sid: &str,
    public_key: Vec<u8>,
    scopes: serde_json::Value,
    ttl_secs: i64,
    signature: Vec<u8>,
) -> sqlx::Result<Asc> {
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + time::Duration::seconds(ttl_secs);

    let row = sqlx::query!(
        r#"
        INSERT INTO id_asc (sid, public_key, scopes, not_before, not_after, signature)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING asc_id, sid, public_key, scopes, not_before, not_after, signature
        "#,
        sid,
        public_key,
        scopes,
        not_before,
        not_after,
        signature
    )
    .fetch_one(pool)
    .await?;

    Ok(Asc {
        asc_id: row.asc_id,
        sid: row.sid,
        public_key: row.public_key,
        scopes: row.scopes,
        not_before: row.not_before,
        not_after: row.not_after,
        signature: row.signature,
    })
}

/// Get active ASC for subject
pub async fn get_active_asc(pool: &PgPool, sid: &str) -> sqlx::Result<Option<Asc>> {
    let now = OffsetDateTime::now_utc();

    let row = sqlx::query!(
        r#"
        SELECT asc_id, sid, public_key, scopes, not_before, not_after, signature
        FROM id_asc
        WHERE sid = $1
          AND not_before <= $2
          AND not_after >= $2
        ORDER BY not_before DESC
        LIMIT 1
        "#,
        sid,
        now
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Asc {
        asc_id: r.asc_id,
        sid: r.sid,
        public_key: r.public_key,
        scopes: r.scopes,
        not_before: r.not_before,
        not_after: r.not_after,
        signature: r.signature,
    }))
}

/// Get ASC by ID (for validation endpoint)
pub async fn get_asc_by_id(pool: &PgPool, asc_id: Uuid) -> sqlx::Result<Option<Asc>> {
    let row = sqlx::query!(
        r#"
        SELECT asc_id, sid, public_key, scopes, not_before, not_after, signature
        FROM id_asc
        WHERE asc_id = $1
        "#,
        asc_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Asc {
        asc_id: r.asc_id,
        sid: r.sid,
        public_key: r.public_key,
        scopes: r.scopes,
        not_before: r.not_before,
        not_after: r.not_after,
        signature: r.signature,
    }))
}

// ============================================================================
// CREDENTIAL OPERATIONS
// ============================================================================

/// Get credential by SID and kind
pub async fn get_credential(
    pool: &PgPool,
    sid: &str,
    credential_kind: &str,
) -> sqlx::Result<Option<Credential>> {
    let row = sqlx::query!(
        r#"
        SELECT id, sid, credential_kind, credential_id, public_key, sign_count, key_version
        FROM id_credential
        WHERE sid = $1 AND credential_kind = $2
        ORDER BY key_version DESC
        LIMIT 1
        "#,
        sid,
        credential_kind
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Credential {
        id: r.id,
        sid: r.sid,
        credential_kind: r.credential_kind,
        credential_id: r.credential_id,
        public_key: r.public_key,
        sign_count: r.sign_count.unwrap_or(0),
        key_version: r.key_version,
    }))
}

/// Update sign_count for passkey
pub async fn update_sign_count(
    pool: &PgPool,
    credential_id: Uuid,
    new_count: i64,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        UPDATE id_credential
        SET sign_count = $1
        WHERE id = $2
        "#,
        new_count,
        credential_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Rotate key (new version + revoke old)
pub async fn rotate_key(
    pool: &PgPool,
    sid: &str,
    new_public_key: Vec<u8>,
    old_key_version: i32,
) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    // Insert new credential
    let new_version = old_key_version + 1;
    sqlx::query!(
        r#"
        INSERT INTO id_credential (sid, credential_kind, public_key, key_version)
        VALUES ($1, 'ed25519', $2, $3)
        "#,
        sid,
        new_public_key,
        new_version
    )
    .execute(&mut *tx)
    .await?;

    // Revoke old key
    sqlx::query!(
        r#"
        INSERT INTO id_key_revocation (sid, key_version)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
        sid,
        old_key_version
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Check if key version is revoked
pub async fn is_key_revoked(
    pool: &PgPool,
    sid: &str,
    key_version: i32,
) -> sqlx::Result<bool> {
    let row = sqlx::query!(
        r#"
        SELECT 1 as exists
        FROM id_key_revocation
        WHERE sid = $1 AND key_version = $2
        "#,
        sid,
        key_version
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

/// Get subject by SID
pub async fn get_subject_by_sid(pool: &PgPool, sid: &str) -> sqlx::Result<Option<Subject>> {
    let row = sqlx::query!(
        r#"
        SELECT sid, kind, display_name, status
        FROM id_subject
        WHERE sid = $1
        "#,
        sid
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Subject {
        sid: r.sid,
        kind: r.kind,
        display_name: r.display_name,
        status: r.status,
    }))
}

/// List all ASCs for a subject
pub async fn list_asc(pool: &PgPool, sid: &str) -> sqlx::Result<Vec<Asc>> {
    let rows = sqlx::query!(
        r#"
        SELECT asc_id, sid, public_key, scopes, not_before, not_after, signature
        FROM id_asc
        WHERE sid = $1
        ORDER BY not_before DESC
        "#,
        sid
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Asc {
            asc_id: r.asc_id,
            sid: r.sid,
            public_key: r.public_key,
            scopes: r.scopes,
            not_before: r.not_before,
            not_after: r.not_after,
            signature: r.signature,
        })
        .collect())
}

/// Revoke ASC (soft delete - mark as expired)
pub async fn revoke_asc(pool: &PgPool, asc_id: Uuid) -> sqlx::Result<()> {
    let now = OffsetDateTime::now_utc();
    
    sqlx::query!(
        r#"
        UPDATE id_asc
        SET not_after = $1
        WHERE asc_id = $2
        "#,
        now,
        asc_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Create ICTE session
pub async fn create_icte_session(
    pool: &PgPool,
    sid: &str,
    scope: serde_json::Value,
    ttl_secs: i64,
) -> sqlx::Result<Uuid> {
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + time::Duration::seconds(ttl_secs);
    let exp_unix = not_after.unix_timestamp();
    let token = Uuid::new_v4().to_string();
    let session_uuid = Uuid::new_v4();

    let row = sqlx::query!(
        r#"
        INSERT INTO id_session (token, sid, session_id, flavor, scope, exp_unix, not_before, not_after)
        VALUES ($1, $2, $3, 'ict', $4, $5, $6, $7)
        RETURNING session_id
        "#,
        token,
        sid,
        session_uuid,
        scope,
        exp_unix,
        not_before,
        not_after
    )
    .fetch_one(pool)
    .await?;

    Ok(row.session_id.unwrap_or(session_uuid))
}

/// Close ICTE session (mark as expired)
pub async fn close_icte_session(pool: &PgPool, session_id: Uuid) -> sqlx::Result<()> {
    let now = OffsetDateTime::now_utc();
    
    sqlx::query!(
        r#"
        UPDATE id_session
        SET not_after = $1
        WHERE session_id = $2 AND flavor = 'ict'
        "#,
        now,
        session_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ============================================================================
// ATOMIC OPERATIONS (Diamond Checklist #4)
// ============================================================================

/// Diamond Checklist #4: Consume challenge and create session atomically
/// This ensures challenge consumption and session creation happen in a single
/// transaction, eliminating the race condition window for replay attacks.
pub async fn consume_challenge_and_create_session(
    pool: &PgPool,
    challenge_id: Uuid,
    expected_origin: &str,
    session_token: &str,
    sid: &str,
    tenant_id: Option<&str>,
    flavor: &str,
    scope: &serde_json::Value,
    exp_unix: i64,
) -> sqlx::Result<Option<Challenge>> {
    // Begin transaction
    let mut tx = pool.begin().await?;
    
    // 1. Atomically consume challenge (UPDATE...WHERE used=false)
    let challenge: Option<ChallengeRow> = sqlx::query_as(
        r#"
        UPDATE id_challenge
        SET used = true
        WHERE id = $1 
          AND used = false 
          AND origin = $2
          AND expires_at > NOW()
        RETURNING id, kind, sid, challenge, origin, expires_at, used
        "#
    )
    .bind(challenge_id)
    .bind(expected_origin)
    .fetch_optional(&mut *tx)
    .await?;
    
    if challenge.is_none() {
        // Challenge not found or already used - rollback and return
        tx.rollback().await?;
        return Ok(None);
    }
    
    // 2. Create session in same transaction
    sqlx::query(
        r#"INSERT INTO id_session (token, sid, tenant_id, flavor, scope, exp_unix)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (token) DO UPDATE 
           SET sid=$2, tenant_id=$3, flavor=$4, scope=$5, exp_unix=$6"#
    )
    .bind(session_token)
    .bind(sid)
    .bind(tenant_id)
    .bind(flavor)
    .bind(scope)
    .bind(exp_unix)
    .execute(&mut *tx)
    .await?;
    
    // 3. Commit transaction
    tx.commit().await?;
    
    // Convert to Challenge
    let row = challenge.unwrap();
    Ok(Some(Challenge {
        id: row.id,
        kind: row.kind,
        sid: row.sid,
        challenge: row.challenge,
        origin: row.origin,
        expires_at: row.expires_at,
        used: row.used,
    }))
}
