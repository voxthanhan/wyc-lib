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
    NumericOverflow,
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

/// Runtime calculation failures preserve contract errors and distinguish
/// arithmetic overflow from a non-finite value supplied by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisError {
    Contract(ContractError),
    NumericOverflow { path: String },
}

impl AnalysisError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Contract(error) => error.code,
            Self::NumericOverflow { .. } => ErrorCode::NumericOverflow,
        }
    }

    pub fn path(&self) -> &str {
        match self {
            Self::Contract(error) => &error.path,
            Self::NumericOverflow { path } => path,
        }
    }
}

impl From<ContractError> for AnalysisError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {}", self.code(), self.path())
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Contract(error) => Some(error),
            Self::NumericOverflow { .. } => None,
        }
    }
}
