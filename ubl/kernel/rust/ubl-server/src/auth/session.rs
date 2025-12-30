use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub sid: Uuid,
    pub tenant_id: Option<String>,  // Zona Schengen: tenant context
    pub flavor: SessionFlavor,
    pub scope: serde_json::Value,
    pub exp_unix: i64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionFlavor {
    Regular,
    #[serde(rename = "stepup")]
    StepUp,
}

impl Session {
    pub fn new_regular(sid: Uuid) -> Self {
        Self::new_regular_with_tenant(sid, None)
    }

    pub fn new_regular_with_tenant(sid: Uuid, tenant_id: Option<String>) -> Self {
        let exp = OffsetDateTime::now_utc() + Duration::hours(1);
        Self {
            token: Uuid::new_v4().to_string(),
            sid,
            tenant_id,
            flavor: SessionFlavor::Regular,
            scope: serde_json::json!({}),
            exp_unix: exp.unix_timestamp(),
        }
    }

    pub fn new_stepup(sid: Uuid) -> Self {
        Self::new_stepup_with_tenant(sid, None)
    }

    pub fn new_stepup_with_tenant(sid: Uuid, tenant_id: Option<String>) -> Self {
        let exp = OffsetDateTime::now_utc() + Duration::minutes(10);
        Self {
            token: Uuid::new_v4().to_string(),
            sid,
            tenant_id,
            flavor: SessionFlavor::StepUp,
            scope: serde_json::json!({"role": "admin"}),
            exp_unix: exp.unix_timestamp(),
        }
    }

    pub fn ttl_secs(&self) -> i64 {
        (self.exp_unix - OffsetDateTime::now_utc().unix_timestamp()).max(0)
    }

    pub fn is_valid(&self) -> bool {
        OffsetDateTime::now_utc().unix_timestamp() < self.exp_unix
    }
}
