use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Subject ID - string format "ubl:sid:<hash>" or legacy UUID
pub type Sid = String;

/// Zona Schengen Context - propagated context that doesn't require re-auth
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionContext {
    /// Current tenant/organization
    pub tenant_id: Option<String>,
    /// Current role within tenant (owner, admin, member)
    pub role: Option<String>,
    /// Operating mode (admin, viewer, readonly)
    pub mode: Option<String>,
    /// Active workspace within tenant
    pub workspace_id: Option<String>,
    /// If impersonating another user (admin feature)
    pub impersonating: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub sid: Sid,  // Subject ID: "ubl:sid:<hash>" format
    pub tenant_id: Option<String>,  // Quick access (also in context)
    pub flavor: SessionFlavor,
    pub scope: serde_json::Value,   // Extended scope for flexibility
    pub context: SessionContext,    // Zona Schengen context
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
    pub fn new_regular(sid: impl Into<Sid>) -> Self {
        Self::new_with_context(sid.into(), SessionFlavor::Regular, SessionContext::default())
    }

    pub fn new_regular_with_tenant(sid: impl Into<Sid>, tenant_id: Option<String>) -> Self {
        let ctx = SessionContext {
            tenant_id: tenant_id.clone(),
            ..Default::default()
        };
        let mut session = Self::new_with_context(sid.into(), SessionFlavor::Regular, ctx);
        session.tenant_id = tenant_id;  // Also set top-level for quick access
        session
    }

    pub fn new_stepup(sid: impl Into<Sid>) -> Self {
        Self::new_stepup_with_tenant(sid.into(), None)
    }

    pub fn new_stepup_with_tenant(sid: impl Into<Sid>, tenant_id: Option<String>) -> Self {
        let sid = sid.into();
        let ctx = SessionContext {
            tenant_id: tenant_id.clone(),
            role: Some("admin".to_string()),  // Step-up implies admin role
            ..Default::default()
        };
        let mut session = Self::new_with_context(sid, SessionFlavor::StepUp, ctx);
        session.tenant_id = tenant_id;
        session.scope = serde_json::json!({"role": "admin"});  // Legacy compat
        session
    }

    /// Create session with full context control
    pub fn new_with_context(sid: Sid, flavor: SessionFlavor, context: SessionContext) -> Self {
        let exp = match flavor {
            SessionFlavor::Regular => OffsetDateTime::now_utc() + Duration::hours(1),
            SessionFlavor::StepUp => OffsetDateTime::now_utc() + Duration::minutes(10),
        };
        Self {
            token: Uuid::new_v4().to_string(),
            sid,
            tenant_id: context.tenant_id.clone(),
            flavor,
            scope: serde_json::json!({}),
            context,
            exp_unix: exp.unix_timestamp(),
        }
    }

    /// Update context without creating new session
    pub fn with_tenant(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id.clone());
        self.context.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.context.role = Some(role);
        self
    }

    pub fn with_mode(mut self, mode: String) -> Self {
        self.context.mode = Some(mode);
        self
    }

    pub fn with_workspace(mut self, workspace_id: String) -> Self {
        self.context.workspace_id = Some(workspace_id);
        self
    }

    pub fn ttl_secs(&self) -> i64 {
        (self.exp_unix - OffsetDateTime::now_utc().unix_timestamp()).max(0)
    }

    pub fn is_valid(&self) -> bool {
        OffsetDateTime::now_utc().unix_timestamp() < self.exp_unix
    }

    /// Check if session has admin privileges (step-up or admin role)
    pub fn is_admin(&self) -> bool {
        self.flavor == SessionFlavor::StepUp || 
        self.context.role.as_deref() == Some("admin") ||
        self.context.role.as_deref() == Some("owner")
    }
}
