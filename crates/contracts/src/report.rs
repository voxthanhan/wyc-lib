use crate::{validate_timestamp, ContractError, ErrorCode, FiniteF64, Module};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Watching,
    Candidate,
    Confirmed,
    Invalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLayer {
    Observation,
    Context,
    Hypothesis,
    Confirmation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub evidence_id: String,
    pub event_type: String,
    pub observed_at: i64,
    pub known_at: i64,
    pub source_module: Module,
    pub measured_values: BTreeMap<String, FiniteF64>,
    pub rule_id: String,
    pub status: ReportStatus,
    pub layer: EvidenceLayer,
}

impl Evidence {
    pub fn validate_at(&self, as_of: i64) -> Result<(), ContractError> {
        validate_timestamp(as_of, "as_of")?;
        validate_timestamp(self.observed_at, "observed_at")?;
        validate_timestamp(self.known_at, "known_at")?;
        if self.observed_at > self.known_at
            || self.known_at > as_of
            || self.evidence_id.trim().is_empty()
            || self.event_type.trim().is_empty()
            || self.rule_id.trim().is_empty()
        {
            return Err(ContractError::new(ErrorCode::InvalidEvidence, "evidence"));
        }
        Ok(())
    }
}

/// Draft P5 shape only: P0 provides serialization, not report evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionReport {
    pub schema_version: String,
    pub report_id: String,
    pub symbol: String,
    pub timeframe: String,
    pub as_of: i64,
    pub engine_version: String,
    pub algorithm_version: String,
    pub data_revision: String,
    pub config_hash: String,
    pub hypothesis: String,
    pub status: ReportStatus,
    pub evidence: Vec<Evidence>,
    pub counter_evidence: Vec<Evidence>,
    pub pending_conditions: Vec<String>,
    pub confirmation: Option<ReportCondition>,
    pub invalidation: Option<ReportCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportCondition {
    pub rule_id: String,
    pub evidence_ids: Vec<String>,
    pub observed_at: Option<i64>,
    pub known_at: Option<i64>,
}
