//! Simulation - Safety Net for Actions
//!
//! Allows LLMs to test actions before executing them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::Affordance;
use crate::Result;

/// Configuration for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Risk threshold requiring simulation
    pub risk_threshold: f32,
    /// Maximum simulation iterations
    pub max_iterations: u32,
    /// Timeout in seconds
    pub timeout_secs: u32,
    /// Enable parallel outcome evaluation
    pub parallel_outcomes: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            risk_threshold: 0.7,
            max_iterations: 10,
            timeout_secs: 30,
            parallel_outcomes: true,
        }
    }
}

/// An action to simulate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action ID (affordance ID)
    pub id: String,
    /// Action name
    pub name: String,
    /// Action parameters
    pub parameters: serde_json::Value,
    /// Risk score (0.0 to 1.0)
    pub risk_score: f32,
    /// Entity performing the action
    pub entity_id: String,
}

/// Possible outcome of an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutcome {
    /// Outcome description
    pub description: String,
    /// Probability (0.0 to 1.0)
    pub probability: f32,
    /// Severity (-1.0 = very bad, 0.0 = neutral, 1.0 = very good)
    pub severity: f32,
    /// Consequences
    pub consequences: Vec<String>,
    /// Is this a terminal state?
    pub is_terminal: bool,
}

/// Recommendation for action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRecommendation {
    /// Proceed with the action
    Proceed,
    /// Modify the action before proceeding
    Modify,
    /// Abandon the action
    Abandon,
    /// Seek human confirmation
    SeekConfirmation,
}

/// Result of simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Action that was simulated
    pub action: Action,
    /// Possible outcomes
    pub outcomes: Vec<ActionOutcome>,
    /// Overall recommendation
    pub recommendation: ActionRecommendation,
    /// Confidence in the recommendation
    pub confidence: f32,
    /// Reasoning for the recommendation
    pub reasoning: String,
    /// Suggested modifications (if recommendation is Modify)
    pub suggested_modifications: Vec<String>,
    /// Simulation timestamp
    pub simulated_at: DateTime<Utc>,
    /// Simulation duration in milliseconds
    pub duration_ms: u64,
}

/// Simulation engine
pub struct Simulation {
    config: SimulationConfig,
}

impl Simulation {
    /// Create a new simulation engine
    pub fn new(config: SimulationConfig) -> Self {
        Self { config }
    }

    /// Check if simulation is required for an action
    pub fn is_required(&self, action: &Action) -> bool {
        action.risk_score >= self.config.risk_threshold
    }

    /// Simulate an action and return results
    pub async fn simulate(&self, action: Action) -> Result<SimulationResult> {
        let start = std::time::Instant::now();

        // Generate possible outcomes based on action type and parameters
        let outcomes = self.generate_outcomes(&action);

        // Calculate recommendation based on outcomes
        let (recommendation, confidence, reasoning) = self.calculate_recommendation(&outcomes);

        // Generate suggested modifications if needed
        let suggested_modifications = if recommendation == ActionRecommendation::Modify {
            self.generate_modifications(&action, &outcomes)
        } else {
            vec![]
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(SimulationResult {
            action,
            outcomes,
            recommendation,
            confidence,
            reasoning,
            suggested_modifications,
            simulated_at: Utc::now(),
            duration_ms,
        })
    }

    /// Generate possible outcomes for an action
    fn generate_outcomes(&self, action: &Action) -> Vec<ActionOutcome> {
        // Default outcomes based on risk level
        let mut outcomes = Vec::new();

        // Success outcome
        outcomes.push(ActionOutcome {
            description: format!("{} completes successfully", action.name),
            probability: 1.0 - action.risk_score * 0.5,
            severity: 0.5,
            consequences: vec![
                "Action completed as intended".to_string(),
                "State updated in ledger".to_string(),
            ],
            is_terminal: true,
        });

        // Partial success
        if action.risk_score > 0.3 {
            outcomes.push(ActionOutcome {
                description: format!("{} partially completes", action.name),
                probability: action.risk_score * 0.3,
                severity: 0.0,
                consequences: vec![
                    "Some aspects completed".to_string(),
                    "Manual intervention may be needed".to_string(),
                ],
                is_terminal: false,
            });
        }

        // Failure outcome
        if action.risk_score > 0.5 {
            outcomes.push(ActionOutcome {
                description: format!("{} fails", action.name),
                probability: action.risk_score * 0.2,
                severity: -0.5,
                consequences: vec![
                    "Action could not be completed".to_string(),
                    "Rollback may be required".to_string(),
                ],
                is_terminal: true,
            });
        }

        // Severe failure
        if action.risk_score > 0.7 {
            outcomes.push(ActionOutcome {
                description: format!("{} causes cascading failure", action.name),
                probability: action.risk_score * 0.1,
                severity: -1.0,
                consequences: vec![
                    "Action failed with side effects".to_string(),
                    "Multiple systems affected".to_string(),
                    "Immediate escalation required".to_string(),
                ],
                is_terminal: true,
            });
        }

        // Normalize probabilities
        let total_prob: f32 = outcomes.iter().map(|o| o.probability).sum();
        for outcome in &mut outcomes {
            outcome.probability /= total_prob;
        }

        outcomes
    }

    /// Calculate recommendation based on outcomes
    fn calculate_recommendation(
        &self,
        outcomes: &[ActionOutcome],
    ) -> (ActionRecommendation, f32, String) {
        // Calculate expected value
        let expected_value: f32 = outcomes.iter()
            .map(|o| o.probability * o.severity)
            .sum();

        // Calculate worst-case probability
        let worst_case_prob: f32 = outcomes.iter()
            .filter(|o| o.severity < -0.5)
            .map(|o| o.probability)
            .sum();

        // Determine recommendation
        let (recommendation, reasoning) = if expected_value > 0.3 && worst_case_prob < 0.1 {
            (ActionRecommendation::Proceed,
             "Expected outcome is positive with low risk of severe failure.")
        } else if expected_value > 0.0 && worst_case_prob < 0.2 {
            (ActionRecommendation::Proceed,
             "Expected outcome is slightly positive. Proceed with caution.")
        } else if expected_value > -0.2 && worst_case_prob < 0.3 {
            (ActionRecommendation::Modify,
             "Action has moderate risk. Consider modifications to reduce risk.")
        } else if worst_case_prob >= 0.3 {
            (ActionRecommendation::SeekConfirmation,
             "Significant probability of severe failure. Seek human confirmation before proceeding.")
        } else {
            (ActionRecommendation::Abandon,
             "Expected outcome is negative. Recommend abandoning this action.")
        };

        // Calculate confidence based on outcome clarity
        let variance: f32 = outcomes.iter()
            .map(|o| (o.severity - expected_value).powi(2) * o.probability)
            .sum();
        let confidence = (1.0 - variance.sqrt()).max(0.5);

        (recommendation, confidence, reasoning.to_string())
    }

    /// Generate modification suggestions
    fn generate_modifications(&self, action: &Action, outcomes: &[ActionOutcome]) -> Vec<String> {
        let mut modifications = Vec::new();

        // General suggestions based on risk
        if action.risk_score > 0.7 {
            modifications.push("Consider breaking this into smaller, lower-risk steps".to_string());
        }

        // Check for high-severity outcomes
        for outcome in outcomes.iter().filter(|o| o.severity < -0.5) {
            modifications.push(format!(
                "Add safeguard against: {}",
                outcome.description
            ));
        }

        // Parameter-based suggestions
        if let Some(obj) = action.parameters.as_object() {
            if obj.contains_key("amount") || obj.contains_key("value") {
                modifications.push("Consider reducing the amount/value".to_string());
            }
            if obj.contains_key("recipients") || obj.contains_key("targets") {
                modifications.push("Consider reducing the number of targets".to_string());
            }
        }

        modifications
    }

    /// Quick check without full simulation
    pub fn quick_check(&self, affordance: &Affordance) -> ActionRecommendation {
        if affordance.risk_score < 0.3 {
            ActionRecommendation::Proceed
        } else if affordance.risk_score < 0.5 {
            ActionRecommendation::Proceed
        } else if affordance.risk_score < 0.7 {
            ActionRecommendation::Modify
        } else if affordance.risk_score < 0.9 {
            ActionRecommendation::SeekConfirmation
        } else {
            ActionRecommendation::Abandon
        }
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new(SimulationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_required() {
        let simulation = Simulation::default();

        let low_risk = Action {
            id: "action_1".to_string(),
            name: "Low Risk".to_string(),
            parameters: serde_json::json!({}),
            risk_score: 0.3,
            entity_id: "entity_1".to_string(),
        };

        let high_risk = Action {
            id: "action_2".to_string(),
            name: "High Risk".to_string(),
            parameters: serde_json::json!({}),
            risk_score: 0.8,
            entity_id: "entity_1".to_string(),
        };

        assert!(!simulation.is_required(&low_risk));
        assert!(simulation.is_required(&high_risk));
    }

    #[tokio::test]
    async fn test_simulate() {
        let simulation = Simulation::default();

        let action = Action {
            id: "action_1".to_string(),
            name: "Test Action".to_string(),
            parameters: serde_json::json!({"amount": 100}),
            risk_score: 0.5,
            entity_id: "entity_1".to_string(),
        };

        let result = simulation.simulate(action).await.unwrap();

        assert!(!result.outcomes.is_empty());
        assert!(result.confidence > 0.0);
        assert!(!result.reasoning.is_empty());
    }

    #[test]
    fn test_quick_check() {
        let simulation = Simulation::default();

        let low_risk = Affordance {
            id: "aff_1".to_string(),
            name: "Low Risk".to_string(),
            description: "".to_string(),
            risk_score: 0.2,
            requires_simulation: false,
            parameters: None,
        };

        let high_risk = Affordance {
            id: "aff_2".to_string(),
            name: "High Risk".to_string(),
            description: "".to_string(),
            risk_score: 0.95,
            requires_simulation: true,
            parameters: None,
        };

        assert_eq!(simulation.quick_check(&low_risk), ActionRecommendation::Proceed);
        assert_eq!(simulation.quick_check(&high_risk), ActionRecommendation::Abandon);
    }
}
