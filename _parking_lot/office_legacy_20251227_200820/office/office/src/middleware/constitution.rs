//! Constitution Enforcer - Office AOP Rules
//!
//! From the principles:
//! > A "Constituição do Office" é um conjunto de regras complementares
//! > (pré-flight, UX, ergonomia, safety extra), nunca substitutas.
//!
//! This module enforces office.constitution.yaml rules:
//! - Max risk levels per mode (operator/admin)
//! - Pre-flight checks (diff required, maintenance windows)
//! - Denylists (blocked job types, targets)
//!
//! "Mais estrito é permitido; mais frouxo é proibido." (Office ⊆ UBL)

use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{Utc, Datelike, Timelike, Weekday};

/// Errors from constitution enforcement
#[derive(Error, Debug)]
pub enum ConstitutionError {
    #[error("Risk level {0} exceeds maximum {1} for mode {2}")]
    RiskLevelExceeded(String, String, String),

    #[error("Job type {0} is blocked by denylist")]
    JobTypeBlocked(String),

    #[error("Target {0} is blocked by denylist")]
    TargetBlocked(String),

    #[error("Pre-flight check failed: {0}")]
    PreFlightFailed(String),

    #[error("Action blocked by maintenance window: {0}")]
    MaintenanceWindow(String),

    #[error("Step-up authentication required for this action")]
    StepUpRequired,
}

/// Office constitution (loaded from office.constitution.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeConstitution {
    pub version: String,
    pub precedence: String,
    pub allow_modes: AllowModes,
    pub pre_flight: PreFlightConfig,
    pub denylists: Denylists,
    pub bindings_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowModes {
    pub operator: ModeConfig,
    pub admin: ModeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub max_risk: String,
    #[serde(default)]
    pub require_step_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreFlightConfig {
    pub require_diff_for: Vec<String>,
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    pub target: String,
    pub days: Vec<String>,
    pub start: String,
    pub end: String,
    pub block: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Denylists {
    pub job_types: Vec<String>,
    pub targets: Vec<String>,
}

impl Default for OfficeConstitution {
    fn default() -> Self {
        Self {
            version: "1.1".to_string(),
            precedence: "UBL > Office".to_string(),
            allow_modes: AllowModes {
                operator: ModeConfig {
                    max_risk: "L2".to_string(),
                    require_step_up: false,
                },
                admin: ModeConfig {
                    max_risk: "L5".to_string(),
                    require_step_up: true,
                },
            },
            pre_flight: PreFlightConfig {
                require_diff_for: vec![
                    "git.registry.merge_protected".to_string(),
                    "system.security.patch".to_string(),
                ],
                maintenance_windows: vec![],
            },
            denylists: Denylists {
                job_types: vec![],
                targets: vec![],
            },
            bindings_required: vec![
                "permit.scopes.tenant_id".to_string(),
                "permit.scopes.jobType".to_string(),
                "permit.scopes.target".to_string(),
                "permit.scopes.subject_hash".to_string(),
                "permit.scopes.policy_hash".to_string(),
            ],
        }
    }
}

/// Constitution enforcer
pub struct ConstitutionEnforcer {
    constitution: OfficeConstitution,
    blocked_job_types: HashSet<String>,
    blocked_targets: HashSet<String>,
}

impl ConstitutionEnforcer {
    /// Create from constitution config
    pub fn new(constitution: OfficeConstitution) -> Self {
        let blocked_job_types: HashSet<String> = constitution.denylists.job_types.iter().cloned().collect();
        let blocked_targets: HashSet<String> = constitution.denylists.targets.iter().cloned().collect();

        Self {
            constitution,
            blocked_job_types,
            blocked_targets,
        }
    }

    /// Create with default constitution
    pub fn default_enforcer() -> Self {
        Self::new(OfficeConstitution::default())
    }

    /// Enforce all constitution rules before requesting permit
    ///
    /// This is AOP: runs BEFORE calling UBL.
    /// Office can only RESTRICT more, never ALLOW more than UBL.
    pub fn enforce(
        &self,
        mode: &str,
        risk_level: &str,
        job_type: &str,
        target: &str,
        has_diff: bool,
        has_step_up: bool,
    ) -> Result<(), ConstitutionError> {
        // 1. Check denylists (Office can always block)
        self.check_denylists(job_type, target)?;

        // 2. Check risk level for mode
        self.check_risk_level(mode, risk_level)?;

        // 3. Check step-up requirement
        self.check_step_up(mode, has_step_up)?;

        // 4. Check pre-flight requirements
        self.check_pre_flight(job_type, has_diff)?;

        // 5. Check maintenance windows
        self.check_maintenance_window(target)?;

        Ok(())
    }

    /// Check if job type or target is blocked
    fn check_denylists(&self, job_type: &str, target: &str) -> Result<(), ConstitutionError> {
        if self.blocked_job_types.contains(job_type) {
            return Err(ConstitutionError::JobTypeBlocked(job_type.to_string()));
        }
        if self.blocked_targets.contains(target) {
            return Err(ConstitutionError::TargetBlocked(target.to_string()));
        }
        Ok(())
    }

    /// Check if risk level is allowed for mode
    fn check_risk_level(&self, mode: &str, risk_level: &str) -> Result<(), ConstitutionError> {
        let max_risk = match mode {
            "operator" => &self.constitution.allow_modes.operator.max_risk,
            "admin" => &self.constitution.allow_modes.admin.max_risk,
            _ => return Err(ConstitutionError::RiskLevelExceeded(
                risk_level.to_string(),
                "unknown".to_string(),
                mode.to_string(),
            )),
        };

        let risk_num = parse_risk_level(risk_level);
        let max_num = parse_risk_level(max_risk);

        if risk_num > max_num {
            return Err(ConstitutionError::RiskLevelExceeded(
                risk_level.to_string(),
                max_risk.clone(),
                mode.to_string(),
            ));
        }

        Ok(())
    }

    /// Check if step-up is required and provided
    fn check_step_up(&self, mode: &str, has_step_up: bool) -> Result<(), ConstitutionError> {
        let requires = match mode {
            "operator" => self.constitution.allow_modes.operator.require_step_up,
            "admin" => self.constitution.allow_modes.admin.require_step_up,
            _ => false,
        };

        if requires && !has_step_up {
            return Err(ConstitutionError::StepUpRequired);
        }

        Ok(())
    }

    /// Check pre-flight requirements
    fn check_pre_flight(&self, job_type: &str, has_diff: bool) -> Result<(), ConstitutionError> {
        if self.constitution.pre_flight.require_diff_for.contains(&job_type.to_string()) {
            if !has_diff {
                return Err(ConstitutionError::PreFlightFailed(
                    format!("Job type {} requires diff preview before execution", job_type)
                ));
            }
        }

        Ok(())
    }

    /// Check maintenance windows
    fn check_maintenance_window(&self, target: &str) -> Result<(), ConstitutionError> {
        let now = Utc::now();
        let current_day = match now.weekday() {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        };
        let current_time = format!("{:02}:{:02}", now.hour(), now.minute());

        for window in &self.constitution.pre_flight.maintenance_windows {
            if window.target != target {
                continue;
            }

            if !window.days.iter().any(|d| d == current_day) {
                continue;
            }

            // Check if current time is within window
            if current_time >= window.start && current_time <= window.end {
                if window.block {
                    return Err(ConstitutionError::MaintenanceWindow(
                        format!(
                            "Target {} is in maintenance window ({} - {}) on {}",
                            target, window.start, window.end, current_day
                        )
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Parse risk level string to number (L0=0, L1=1, ..., L5=5)
fn parse_risk_level(level: &str) -> u8 {
    match level {
        "L0" => 0,
        "L1" => 1,
        "L2" => 2,
        "L3" => 3,
        "L4" => 4,
        "L5" => 5,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_enforcement() {
        let enforcer = ConstitutionEnforcer::default_enforcer();

        // Operator can do L2
        assert!(enforcer.check_risk_level("operator", "L2").is_ok());

        // Operator cannot do L3
        assert!(enforcer.check_risk_level("operator", "L3").is_err());

        // Admin can do L5
        assert!(enforcer.check_risk_level("admin", "L5").is_ok());
    }

    #[test]
    fn test_denylist() {
        let mut constitution = OfficeConstitution::default();
        constitution.denylists.job_types = vec!["dangerous.job".to_string()];
        constitution.denylists.targets = vec!["forbidden_target".to_string()];

        let enforcer = ConstitutionEnforcer::new(constitution);

        assert!(enforcer.check_denylists("dangerous.job", "LAB_512").is_err());
        assert!(enforcer.check_denylists("safe.job", "forbidden_target").is_err());
        assert!(enforcer.check_denylists("safe.job", "LAB_512").is_ok());
    }

    #[test]
    fn test_pre_flight_diff_required() {
        let enforcer = ConstitutionEnforcer::default_enforcer();

        // merge_protected requires diff
        assert!(enforcer.check_pre_flight("git.registry.merge_protected", false).is_err());
        assert!(enforcer.check_pre_flight("git.registry.merge_protected", true).is_ok());

        // regular job doesn't require diff
        assert!(enforcer.check_pre_flight("service.restart", false).is_ok());
    }

    #[test]
    fn test_full_enforcement() {
        let enforcer = ConstitutionEnforcer::default_enforcer();

        // Valid operator action
        let result = enforcer.enforce(
            "operator",
            "L2",
            "service.restart",
            "LAB_256",
            false,
            false,
        );
        assert!(result.is_ok());

        // Operator trying L3 action
        let result = enforcer.enforce(
            "operator",
            "L3",
            "service.restart",
            "LAB_256",
            false,
            false,
        );
        assert!(result.is_err());
    }
}

