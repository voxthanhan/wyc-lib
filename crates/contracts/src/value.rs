use crate::{ContractError, ErrorCode};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A finite f64, enforced at construction and at the JSON boundary.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteF64(f64);

pub(crate) const TWO: FiniteF64 = FiniteF64(2.0);

impl FiniteF64 {
    pub fn new(value: f64) -> Result<Self, ContractError> {
        if value.is_finite() {
            // Normalize negative zero at the wire boundary.
            Ok(Self(if value == 0.0 { 0.0 } else { value }))
        } else {
            Err(ContractError::new(ErrorCode::NonFinite, "value"))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl Serialize for FiniteF64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for FiniteF64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(f64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum NumericValue {
    Ready { value: FiniteF64 },
    Warmup { required: usize, available: usize },
    Missing { reason: MissingReason },
}

impl NumericValue {
    pub fn validate(&self) -> Result<(), ContractError> {
        if let Self::Warmup {
            required,
            available,
        } = self
        {
            if *required == 0 || available >= required {
                return Err(ContractError::new(
                    ErrorCode::InvalidResult,
                    "result.warmup",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingReason {
    ZeroDenominator,
    UnavailableDependency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericSample {
    pub observed_at: i64,
    pub known_at: i64,
    pub result: NumericValue,
}

impl NumericSample {
    pub fn validate_at(&self, as_of: i64) -> Result<(), ContractError> {
        crate::validate_timestamp(as_of, "as_of")?;
        crate::validate_timestamp(self.observed_at, "observed_at")?;
        crate::validate_timestamp(self.known_at, "known_at")?;
        if self.observed_at > self.known_at || self.known_at > as_of {
            return Err(ContractError::new(ErrorCode::InvalidResult, "sample"));
        }
        self.result.validate()
    }
}
