//! Sanity Check - Claim vs Fact Validation
//!
//! Validates claims from handovers against objective facts from the ledger.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::EntityId;
use crate::ubl_client::UblClient;
use crate::Result;

/// Configuration for sanity check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanityCheckConfig {
    /// Enable keyword-based extraction
    pub keyword_extraction: bool,
    /// Keywords to look for
    pub keywords: Vec<String>,
    /// Enable LLM-based extraction (future)
    pub llm_extraction: bool,
    /// Maximum days to look back for facts
    pub lookback_days: u32,
}

impl Default for SanityCheckConfig {
    fn default() -> Self {
        Self {
            keyword_extraction: true,
            keywords: vec![
                "malicioso".to_string(), "malicious".to_string(),
                "insatisfeito".to_string(), "unsatisfied".to_string(),
                "urgente".to_string(), "urgent".to_string(),
                "crítico".to_string(), "critical".to_string(),
                "suspeito".to_string(), "suspicious".to_string(),
                "preocupante".to_string(), "concerning".to_string(),
                "falha".to_string(), "failure".to_string(),
                "erro".to_string(), "error".to_string(),
                "problema".to_string(), "problem".to_string(),
                "atrasado".to_string(), "delayed".to_string(),
            ],
            llm_extraction: false,
            lookback_days: 30,
        }
    }
}

/// A claim extracted from handover
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// The claim text
    pub text: String,
    /// Keywords found
    pub keywords: Vec<String>,
    /// Sentiment (-1.0 to 1.0)
    pub sentiment: f32,
    /// Whether this is a factual claim (vs opinion)
    pub is_factual: bool,
}

/// An objective fact from the ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Fact description
    pub description: String,
    /// Source event ID
    pub source_event_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Category
    pub category: String,
}

/// Discrepancy between claim and fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    /// The claim
    pub claim: Claim,
    /// Contradicting facts
    pub contradicting_facts: Vec<Fact>,
    /// Severity (0.0 to 1.0)
    pub severity: f32,
    /// Suggested governance note
    pub governance_note: String,
}

/// A governance note to inject into narrative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceNote {
    /// Note content
    pub content: String,
    /// Severity
    pub severity: f32,
    /// Source (what triggered this note)
    pub source: String,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Sanity Check - Validates claims against facts
pub struct SanityCheck {
    config: SanityCheckConfig,
    ubl_client: Option<Arc<UblClient>>,
}

impl SanityCheck {
    /// Create a new sanity check
    pub fn new(config: SanityCheckConfig) -> Self {
        Self {
            config,
            ubl_client: None,
        }
    }

    /// Create with UBL client for fact checking
    pub fn with_ubl_client(mut self, client: Arc<UblClient>) -> Self {
        self.ubl_client = Some(client);
        self
    }

    /// Check a handover and generate governance notes
    pub async fn check(&self, handover: &str, entity_id: &EntityId) -> Result<Vec<String>> {
        let claims = self.extract_claims(handover);

        if claims.is_empty() {
            return Ok(vec![]);
        }

        let facts = self.query_facts(entity_id).await?;
        let discrepancies = self.find_discrepancies(&claims, &facts);

        let notes: Vec<String> = discrepancies
            .into_iter()
            .map(|d| d.governance_note)
            .collect();

        Ok(notes)
    }

    /// Extract claims from handover text
    fn extract_claims(&self, handover: &str) -> Vec<Claim> {
        if !self.config.keyword_extraction {
            return vec![];
        }

        let handover_lower = handover.to_lowercase();
        let mut claims = Vec::new();

        // Find sentences containing keywords
        for sentence in handover.split(&['.', '!', '?'][..]) {
            let sentence_lower = sentence.to_lowercase();
            let found_keywords: Vec<String> = self.config.keywords
                .iter()
                .filter(|k| sentence_lower.contains(k.as_str()))
                .cloned()
                .collect();

            if !found_keywords.is_empty() {
                let sentiment = self.estimate_sentiment(&sentence_lower, &found_keywords);
                claims.push(Claim {
                    text: sentence.trim().to_string(),
                    keywords: found_keywords,
                    sentiment,
                    is_factual: self.is_factual_claim(sentence),
                });
            }
        }

        claims
    }

    /// Estimate sentiment based on keywords
    fn estimate_sentiment(&self, text: &str, keywords: &[String]) -> f32 {
        let negative_keywords = ["malicioso", "malicious", "insatisfeito", "unsatisfied",
            "suspeito", "suspicious", "preocupante", "concerning", "falha", "failure",
            "erro", "error", "problema", "problem"];

        let negative_count = keywords.iter()
            .filter(|k| negative_keywords.contains(&k.as_str()))
            .count();

        let total = keywords.len().max(1) as f32;
        let negative_ratio = negative_count as f32 / total;

        // -1.0 = very negative, 0.0 = neutral, 1.0 = positive
        -negative_ratio
    }

    /// Check if a claim is factual (vs opinion)
    fn is_factual_claim(&self, sentence: &str) -> bool {
        let opinion_markers = ["i think", "i believe", "i feel", "seems", "appears",
            "penso", "acho", "sinto", "parece"];

        let sentence_lower = sentence.to_lowercase();
        !opinion_markers.iter().any(|m| sentence_lower.contains(m))
    }

    /// Query objective facts from UBL
    async fn query_facts(&self, entity_id: &EntityId) -> Result<Vec<Fact>> {
        if let Some(client) = &self.ubl_client {
            let events = client.get_events(entity_id, 100).await?;

            let facts: Vec<Fact> = events.into_iter().map(|e| {
                Fact {
                    description: e.summary,
                    source_event_id: e.entry_hash,
                    timestamp: e.timestamp,
                    category: e.intent_class,
                }
            }).collect();

            Ok(facts)
        } else {
            // Without UBL client, return empty facts
            Ok(vec![])
        }
    }

    /// Find discrepancies between claims and facts
    fn find_discrepancies(&self, claims: &[Claim], facts: &[Fact]) -> Vec<Discrepancy> {
        let mut discrepancies = Vec::new();

        for claim in claims {
            // Look for contradicting facts
            let contradicting: Vec<Fact> = facts.iter()
                .filter(|f| self.contradicts(&claim.text, &f.description))
                .cloned()
                .collect();

            if !contradicting.is_empty() {
                let severity = (claim.sentiment.abs() + 0.5).min(1.0);
                let governance_note = self.generate_governance_note(claim, &contradicting);

                discrepancies.push(Discrepancy {
                    claim: claim.clone(),
                    contradicting_facts: contradicting,
                    severity,
                    governance_note,
                });
            } else if claim.sentiment < -0.5 && claim.is_factual {
                // Strong negative claim without contradicting evidence - note for caution
                discrepancies.push(Discrepancy {
                    claim: claim.clone(),
                    contradicting_facts: vec![],
                    severity: 0.3,
                    governance_note: format!(
                        "GOVERNANCE NOTE: The previous handover claims '{}'. \
                        This claim could not be verified against objective records. \
                        Consider verifying before acting on this information.",
                        claim.text
                    ),
                });
            }
        }

        discrepancies
    }

    /// Check if a claim is contradicted by a fact
    fn contradicts(&self, claim: &str, fact: &str) -> bool {
        // Simple heuristic: check for opposite sentiment keywords
        let claim_lower = claim.to_lowercase();
        let fact_lower = fact.to_lowercase();

        // If claim says "delayed" but fact says "on time"
        if (claim_lower.contains("delayed") || claim_lower.contains("atrasado")) &&
           (fact_lower.contains("on time") || fact_lower.contains("completed")) {
            return true;
        }

        // If claim says "failure" but fact says "success"
        if (claim_lower.contains("failure") || claim_lower.contains("falha")) &&
           (fact_lower.contains("success") || fact_lower.contains("sucesso")) {
            return true;
        }

        // If claim says "unsatisfied" but fact says "positive feedback"
        if (claim_lower.contains("unsatisfied") || claim_lower.contains("insatisfeito")) &&
           (fact_lower.contains("positive") || fact_lower.contains("satisfied")) {
            return true;
        }

        false
    }

    /// Generate governance note from discrepancy
    fn generate_governance_note(&self, claim: &Claim, facts: &[Fact]) -> String {
        let fact_summaries: Vec<String> = facts.iter()
            .map(|f| format!("- {}: {}", f.timestamp.format("%Y-%m-%d"), f.description))
            .collect();

        format!(
            "GOVERNANCE NOTE: The previous handover stated '{}', but objective records show:\n{}\n\
            Please verify the current situation before acting on the handover claims.",
            claim.text,
            fact_summaries.join("\n")
        )
    }
}

impl Default for SanityCheck {
    fn default() -> Self {
        Self::new(SanityCheckConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_extraction() {
        let sanity_check = SanityCheck::default();

        let handover = "The client seems suspicious and there's an urgent problem to address.";
        let claims = sanity_check.extract_claims(handover);

        assert!(!claims.is_empty());
        assert!(claims[0].keywords.contains(&"suspicious".to_string()));
    }

    #[test]
    fn test_sentiment_estimation() {
        let sanity_check = SanityCheck::default();

        let negative_keywords = vec!["malicious".to_string(), "failure".to_string()];
        let sentiment = sanity_check.estimate_sentiment("test", &negative_keywords);

        assert!(sentiment < 0.0);
    }

    #[test]
    fn test_factual_claim_detection() {
        let sanity_check = SanityCheck::default();

        assert!(sanity_check.is_factual_claim("The payment was delayed."));
        assert!(!sanity_check.is_factual_claim("I think the payment was delayed."));
    }

    #[tokio::test]
    async fn test_check_without_ubl() {
        let sanity_check = SanityCheck::default();

        let handover = "Everything went smoothly with no issues.";
        let notes = sanity_check.check(handover, &"entity_1".to_string()).await.unwrap();

        assert!(notes.is_empty());
    }
}
