use sqlx::PgPool;
use sqlx::Row;
use crate::auth::session::{Session, SessionFlavor, SessionContext};

pub async fn insert(pool: &PgPool, s: &Session) -> sqlx::Result<()> {
    // sid is already a String (Sid type)
    let sid_str = &s.sid;
    
    // Serialize context into scope for storage
    let scope_with_context = serde_json::json!({
        "legacy": s.scope,
        "context": s.context,
    });
    
    sqlx::query(
        r#"INSERT INTO id_session (token, sid, tenant_id, flavor, scope, exp_unix)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (token) DO UPDATE 
           SET sid=$2, tenant_id=$3, flavor=$4, scope=$5, exp_unix=$6"#
    )
    .bind(&s.token)
    .bind(&sid_str)
    .bind(&s.tenant_id)
    .bind(flavor_str(s.flavor))
    .bind(&scope_with_context)
    .bind(s.exp_unix)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_valid(pool: &PgPool, token: &str) -> sqlx::Result<Option<Session>> {
    let row = sqlx::query(
        r#"SELECT token, sid, tenant_id, flavor, scope, exp_unix
           FROM id_session 
           WHERE token = $1 
             AND exp_unix > EXTRACT(EPOCH FROM now())"#
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|r| {
        let token: String = r.get("token");
        let sid: String = r.get("sid");  // SID is a string, not UUID
        let tenant_id: Option<String> = r.get("tenant_id");
        let flavor: String = r.get("flavor");
        let scope: serde_json::Value = r.get("scope");
        let exp_unix: Option<i64> = r.get("exp_unix");
        
        // Extract context from scope (with fallback for legacy sessions)
        let context: SessionContext = scope.get("context")
            .and_then(|c| serde_json::from_value(c.clone()).ok())
            .unwrap_or_else(|| SessionContext {
                tenant_id: tenant_id.clone(),
                ..Default::default()
            });
        
        // Extract legacy scope if present
        let legacy_scope = scope.get("legacy")
            .cloned()
            .unwrap_or_else(|| scope.clone());
        
        Some(Session {
            token,
            sid,
            tenant_id,
            flavor: match flavor.as_str() {
                "stepup" => SessionFlavor::StepUp,
                _ => SessionFlavor::Regular,
            },
            scope: legacy_scope,
            context,
            exp_unix: exp_unix?,
        })
    }))
}

/// Update session context without invalidating session
pub async fn update_context(pool: &PgPool, token: &str, context: &SessionContext) -> sqlx::Result<()> {
    // First get the current session to preserve other fields
    let session = get_valid(pool, token).await?;
    
    if let Some(mut s) = session {
        s.context = context.clone();
        s.tenant_id = context.tenant_id.clone();
        insert(pool, &s).await?;
    }
    
    Ok(())
}

pub async fn delete(pool: &PgPool, token: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM id_session WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

fn flavor_str(f: SessionFlavor) -> &'static str {
    match f {
        SessionFlavor::StepUp => "stepup",
        SessionFlavor::Regular => "regular",
    }
}
