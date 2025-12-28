use sqlx::PgPool;
use uuid::Uuid;
use crate::auth::session::{Session, SessionFlavor};

pub async fn insert(pool: &PgPool, s: &Session) -> sqlx::Result<()> {
    let sid_str = s.sid.to_string();
    sqlx::query!(
        r#"INSERT INTO id_session (token, sid, flavor, scope, exp_unix)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (token) DO UPDATE 
           SET sid=$2, flavor=$3, scope=$4, exp_unix=$5"#,
        s.token,
        sid_str,
        flavor_str(s.flavor),
        s.scope,
        s.exp_unix
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_valid(pool: &PgPool, token: &str) -> sqlx::Result<Option<Session>> {
    let r = sqlx::query!(
        r#"SELECT token, sid, flavor, scope, exp_unix
           FROM id_session 
           WHERE token = $1 
             AND exp_unix > EXTRACT(EPOCH FROM now())"#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(r.and_then(|x| {
        let sid = Uuid::parse_str(&x.sid).ok()?;
        Some(Session {
            token: x.token,
            sid,
            flavor: match x.flavor.as_str() {
                "stepup" => SessionFlavor::StepUp,
                _ => SessionFlavor::Regular,
            },
            scope: x.scope,
            exp_unix: x.exp_unix?,
        })
    }))
}

pub async fn delete(pool: &PgPool, token: &str) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM id_session WHERE token = $1", token)
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
