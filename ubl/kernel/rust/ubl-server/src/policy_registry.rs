//! Policy Registry - Maps containers to policies
//!
//! SPEC-UBL-POLICY v1.0 §10:
//! > Cada commit referencia explicitamente:
//! > - versão da política aplicada
//! > - hash da política compilada
//!
//! This module manages which policy applies to which container.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::PgPool;
use tracing::{info, error, warn};

use ubl_policy_vm::{
    PolicyVM, PolicyDefinition, CompiledPolicy, EvaluationContext,
    TranslationDecision, PolicyError, create_default_policy,
};

/// Policy registry error
#[derive(Debug)]
pub enum RegistryError {
    PolicyNotFound(String),
    ContainerNotConfigured(String),
    EvaluationFailed(String),
    DatabaseError(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolicyNotFound(id) => write!(f, "Policy not found: {}", id),
            Self::ContainerNotConfigured(id) => write!(f, "No policy configured for container: {}", id),
            Self::EvaluationFailed(e) => write!(f, "Policy evaluation failed: {}", e),
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Container policy mapping
#[derive(Debug, Clone)]
pub struct ContainerPolicy {
    pub container_id: String,
    pub policy_id: String,
    pub policy_version: String,
}

/// Policy registry - manages container -> policy mappings
pub struct PolicyRegistry {
    /// Policy VM for evaluation
    vm: Arc<RwLock<PolicyVM>>,
    /// Container -> Policy ID mapping
    container_policies: RwLock<HashMap<String, String>>,
    /// Database pool for persistence (optional)
    pool: Option<PgPool>,
}

impl PolicyRegistry {
    /// Create a new policy registry
    pub fn new() -> Self {
        Self {
            vm: Arc::new(RwLock::new(PolicyVM::new())),
            container_policies: RwLock::new(HashMap::new()),
            pool: None,
        }
    }

    /// Create with database backing
    pub fn with_pool(pool: PgPool) -> Self {
        Self {
            vm: Arc::new(RwLock::new(PolicyVM::new())),
            container_policies: RwLock::new(HashMap::new()),
            pool: Some(pool),
        }
    }

    /// Initialize default policies for known containers
    pub async fn init_defaults(&self) {
        let known_containers = vec![
            "C.Jobs",
            "C.Messenger",
            "C.Artifacts",
            "C.Pacts",
            "C.Policy",
        ];

        let mut vm = self.vm.write().await;
        let mut mappings = self.container_policies.write().await;

        for container_id in known_containers {
            let definition = create_default_policy(container_id);
            let policy_id = definition.policy_id.clone();
            
            vm.register(&definition);
            mappings.insert(container_id.to_string(), policy_id.clone());
            
            info!("📋 Registered default policy for {}: {}", container_id, policy_id);
        }
    }

    /// Register a policy
    pub async fn register_policy(&self, definition: PolicyDefinition) -> Result<String, RegistryError> {
        let policy_id = definition.policy_id.clone();
        
        let mut vm = self.vm.write().await;
        vm.register(&definition);
        
        // Persist to database if available
        if let Some(ref pool) = self.pool {
            if let Err(e) = self.persist_policy(pool, &definition).await {
                warn!("Failed to persist policy {}: {}", policy_id, e);
            }
        }
        
        info!("📋 Registered policy: {} v{}", policy_id, definition.version);
        Ok(policy_id)
    }

    /// Set policy for a container
    pub async fn set_container_policy(
        &self,
        container_id: &str,
        policy_id: &str,
    ) -> Result<(), RegistryError> {
        // Verify policy exists
        {
            let vm = self.vm.read().await;
            if !vm.has_policy(policy_id) {
                return Err(RegistryError::PolicyNotFound(policy_id.to_string()));
            }
        }

        // Update mapping
        {
            let mut mappings = self.container_policies.write().await;
            mappings.insert(container_id.to_string(), policy_id.to_string());
        }

        // Persist to database if available
        if let Some(ref pool) = self.pool {
            if let Err(e) = self.persist_container_mapping(pool, container_id, policy_id).await {
                warn!("Failed to persist container mapping: {}", e);
            }
        }

        info!("📋 Container {} now uses policy {}", container_id, policy_id);
        Ok(())
    }

    /// Get the policy ID for a container
    pub async fn get_policy_for_container(&self, container_id: &str) -> Option<String> {
        let mappings = self.container_policies.read().await;
        mappings.get(container_id).cloned()
    }

    /// Evaluate policy for a container
    pub async fn evaluate(
        &self,
        container_id: &str,
        actor: &str,
        intent: &serde_json::Value,
        state: Option<serde_json::Value>,
        timestamp: i64,
    ) -> Result<TranslationDecision, RegistryError> {
        // Get policy ID for container
        let policy_id = {
            let mappings = self.container_policies.read().await;
            mappings.get(container_id).cloned()
        };

        let policy_id = match policy_id {
            Some(id) => id,
            None => {
                // No policy configured - use a permissive default (allow Observation)
                warn!("⚠️  No policy for container {}, using permissive default", container_id);
                return Ok(TranslationDecision::Allow {
                    intent_class: 0x00,
                    required_pact: None,
                    constraints: vec![],
                });
            }
        };

        // Build context
        let context = EvaluationContext {
            container_id: container_id.to_string(),
            actor: actor.to_string(),
            intent: intent.clone(),
            state,
            timestamp,
        };

        // Evaluate
        let vm = self.vm.read().await;
        vm.evaluate(&policy_id, &context)
            .map_err(|e| RegistryError::EvaluationFailed(e.to_string()))
    }

    /// Check if policy evaluation is required for an intent class
    pub fn requires_policy_evaluation(intent_class: &str) -> bool {
        // All intents should be evaluated, but Evolution is critical
        match intent_class {
            "Evolution" => true,
            "Entropy" => true,
            _ => true, // For now, evaluate all
        }
    }

    /// Persist policy to database (internal)
    async fn persist_policy(&self, pool: &PgPool, definition: &PolicyDefinition) -> Result<(), sqlx::Error> {
        // Serialize rules
        let rules_json = serde_json::to_value(&definition.rules).unwrap_or(serde_json::Value::Null);
        
        sqlx::query!(
            r#"
            INSERT INTO policy_definitions (policy_id, version, description, rules, default_deny)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (policy_id) DO UPDATE SET
                version = EXCLUDED.version,
                description = EXCLUDED.description,
                rules = EXCLUDED.rules,
                default_deny = EXCLUDED.default_deny,
                updated_at = NOW()
            "#,
            &definition.policy_id,
            &definition.version,
            &definition.description,
            rules_json,
            definition.default_deny,
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }

    /// Persist container mapping to database (internal)
    async fn persist_container_mapping(
        &self,
        pool: &PgPool,
        container_id: &str,
        policy_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO container_policies (container_id, policy_id)
            VALUES ($1, $2)
            ON CONFLICT (container_id) DO UPDATE SET
                policy_id = EXCLUDED.policy_id,
                updated_at = NOW()
            "#,
            container_id,
            policy_id,
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }

    /// Load policies from database on startup
    pub async fn load_from_database(&self) -> Result<(), RegistryError> {
        let Some(ref pool) = self.pool else {
            return Ok(()); // No database configured
        };

        // Load policy definitions
        let policies: Vec<PolicyDefRow> = sqlx::query_as!(
            PolicyDefRow,
            r#"
            SELECT policy_id, version, description, rules, default_deny
            FROM policy_definitions
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        let mut vm = self.vm.write().await;
        for row in policies {
            let rules: Vec<ubl_policy_vm::PolicyRule> = serde_json::from_value(row.rules)
                .unwrap_or_default();
            
            let definition = PolicyDefinition {
                policy_id: row.policy_id.clone(),
                version: row.version,
                description: row.description,
                rules,
                default_deny: row.default_deny,
            };
            
            vm.register(&definition);
            info!("📋 Loaded policy from DB: {}", row.policy_id);
        }

        // Load container mappings
        let mappings: Vec<ContainerMappingRow> = sqlx::query_as!(
            ContainerMappingRow,
            r#"
            SELECT container_id, policy_id
            FROM container_policies
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        let mut container_policies = self.container_policies.write().await;
        for row in mappings {
            container_policies.insert(row.container_id.clone(), row.policy_id.clone());
            info!("📋 Loaded container mapping: {} -> {}", row.container_id, row.policy_id);
        }

        Ok(())
    }
}

impl Default for PolicyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct PolicyDefRow {
    policy_id: String,
    version: String,
    description: String,
    rules: serde_json::Value,
    default_deny: bool,
}

#[derive(Debug)]
struct ContainerMappingRow {
    container_id: String,
    policy_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_init_defaults() {
        let registry = PolicyRegistry::new();
        registry.init_defaults().await;

        let policy_id = registry.get_policy_for_container("C.Jobs").await;
        assert!(policy_id.is_some());
    }

    #[tokio::test]
    async fn test_evaluate_default_policy() {
        let registry = PolicyRegistry::new();
        registry.init_defaults().await;

        let result = registry.evaluate(
            "C.Jobs",
            "alice",
            &json!({"type": "observe"}),
            None,
            1000,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_policy_configured() {
        let registry = PolicyRegistry::new();
        // Don't init defaults

        let result = registry.evaluate(
            "C.Unknown",
            "alice",
            &json!({"type": "observe"}),
            None,
            1000,
        ).await;

        // Should return permissive default
        assert!(matches!(result, Ok(TranslationDecision::Allow { .. })));
    }
}






