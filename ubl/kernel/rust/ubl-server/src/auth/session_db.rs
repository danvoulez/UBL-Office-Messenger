use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;
use crate::auth::session::{Session, SessionFlavor};

pub async fn insert(pool: &PgPool, s: &Session) -> sqlx::Result<()> {
    let sid_str = s.sid.to_string();
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
    .bind(&s.scope)
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
        let sid_str: String = r.get("sid");
        let tenant_id: Option<String> = r.get("tenant_id");
        let flavor: String = r.get("flavor");
        let scope: serde_json::Value = r.get("scope");
        let exp_unix: Option<i64> = r.get("exp_unix");
        
        let sid = Uuid::parse_str(&sid_str).ok()?;
        Some(Session {
            token,
            sid,
            tenant_id,
            flavor: match flavor.as_str() {
                "stepup" => SessionFlavor::StepUp,
                _ => SessionFlavor::Regular,
            },
            scope,
            exp_unix: exp_unix?,
        })
    }))
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
