use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UnsupportedVersion,
    InvalidMetadata,
    LengthMismatch,
    InvalidTimestamp,
    DuplicateTimestamp,
    UnorderedTimestamp,
    InvalidAvailability,
    MissingData,
    NonFinite,
    InvalidOhlc,
    NegativeVolume,
    DeclaredGap,
    RevisionNotAvailable,
    InvalidConfig,
    DuplicateModule,
    MissingDependency,
    UnsupportedModule,
    StateMismatch,
    ResetRequired,
    InvalidEvidence,
    InvalidResult,
}

/// Stable machine-readable code and path; diagnostic text is not an API key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractError {
    pub code: ErrorCode,
    pub path: String,
}

impl ContractError {
    pub fn new(code: ErrorCode, path: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
        }
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {}", self.code, self.path)
    }
}

impl std::error::Error for ContractError {}
