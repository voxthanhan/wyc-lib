use crate::{ContractError, ErrorCode, FiniteF64, SCHEMA_VERSION};
use serde::{Deserialize, Serialize};

/// Nonnegative Unix milliseconds, bounded for exact JavaScript number transport.
pub const MAX_TIMESTAMP: i64 = 9_007_199_254_740_991;

pub fn validate_timestamp(value: i64, path: &str) -> Result<(), ContractError> {
    if (0..=MAX_TIMESTAMP).contains(&value) {
        Ok(())
    } else {
        Err(ContractError::new(ErrorCode::InvalidTimestamp, path))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentMode {
    Raw,
    SplitAdjusted,
    TotalReturnAdjusted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimestampSemantics {
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    PointInTime,
    RevisedHistory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataGap {
    pub after_timestamp: i64,
    pub before_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketMetadata {
    pub source: String,
    pub timezone: String,
    pub price_unit: String,
    pub volume_unit: String,
    pub adjustment_mode: AdjustmentMode,
    pub timestamp_semantics: TimestampSemantics,
    pub data_revision: String,
    pub revision_available_at: i64,
    pub snapshot_kind: SnapshotKind,
    pub gaps: Vec<DataGap>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketData {
    pub schema_version: String,
    pub symbol: String,
    pub timeframe: String,
    pub timestamp: Vec<i64>,
    pub available_at: Vec<i64>,
    pub open: Vec<Option<FiniteF64>>,
    pub high: Vec<Option<FiniteF64>>,
    pub low: Vec<Option<FiniteF64>>,
    pub close: Vec<Option<FiniteF64>>,
    pub volume: Vec<Option<FiniteF64>>,
    pub is_closed: Vec<bool>,
    pub metadata: MarketMetadata,
}

impl MarketData {
    /// Validate the entire snapshot without sorting, repairing or mutating it.
    pub fn validate(&self) -> Result<(), ContractError> {
        use ErrorCode::*;
        if self.schema_version != SCHEMA_VERSION {
            return Err(ContractError::new(UnsupportedVersion, "schema_version"));
        }
        for (path, value) in [
            ("symbol", &self.symbol),
            ("timeframe", &self.timeframe),
            ("metadata.source", &self.metadata.source),
            ("metadata.timezone", &self.metadata.timezone),
            ("metadata.price_unit", &self.metadata.price_unit),
            ("metadata.volume_unit", &self.metadata.volume_unit),
            ("metadata.data_revision", &self.metadata.data_revision),
        ] {
            if value.trim().is_empty() {
                return Err(ContractError::new(InvalidMetadata, path));
            }
        }
        validate_timestamp(
            self.metadata.revision_available_at,
            "metadata.revision_available_at",
        )?;
        let n = self.timestamp.len();
        for (path, len) in [
            ("available_at", self.available_at.len()),
            ("open", self.open.len()),
            ("high", self.high.len()),
            ("low", self.low.len()),
            ("close", self.close.len()),
            ("volume", self.volume.len()),
            ("is_closed", self.is_closed.len()),
        ] {
            if len != n {
                return Err(ContractError::new(LengthMismatch, path));
            }
        }
        for i in 0..n {
            validate_timestamp(self.timestamp[i], &format!("timestamp[{i}]"))?;
            validate_timestamp(self.available_at[i], &format!("available_at[{i}]"))?;
            if i > 0 && self.timestamp[i] <= self.timestamp[i - 1] {
                let code = if self.timestamp[i] == self.timestamp[i - 1] {
                    DuplicateTimestamp
                } else {
                    UnorderedTimestamp
                };
                return Err(ContractError::new(code, format!("timestamp[{i}]")));
            }
            if self.available_at[i] < self.timestamp[i]
                || (i > 0 && self.available_at[i] < self.available_at[i - 1])
                || (i > 0 && self.is_closed[i] && !self.is_closed[i - 1])
            {
                return Err(ContractError::new(
                    InvalidAvailability,
                    format!("available_at[{i}]"),
                ));
            }
            let mut values = [0.0; 5];
            for (j, (path, column)) in [
                ("open", &self.open),
                ("high", &self.high),
                ("low", &self.low),
                ("close", &self.close),
                ("volume", &self.volume),
            ]
            .into_iter()
            .enumerate()
            {
                values[j] = column[i]
                    .ok_or_else(|| ContractError::new(MissingData, format!("{path}[{i}]")))?
                    .get();
            }
            let [open, high, low, close, volume] = values;
            if low > high || open < low || open > high || close < low || close > high {
                return Err(ContractError::new(InvalidOhlc, format!("ohlc[{i}]")));
            }
            if volume < 0.0 {
                return Err(ContractError::new(NegativeVolume, format!("volume[{i}]")));
            }
        }
        // Even valid declared gaps are refused by the initial continuous-series contract.
        for gap in &self.metadata.gaps {
            validate_timestamp(gap.after_timestamp, "metadata.gaps.after_timestamp")?;
            validate_timestamp(gap.before_timestamp, "metadata.gaps.before_timestamp")?;
            if gap.after_timestamp >= gap.before_timestamp {
                return Err(ContractError::new(InvalidMetadata, "metadata.gaps"));
            }
        }
        if !self.metadata.gaps.is_empty() {
            return Err(ContractError::new(DeclaredGap, "metadata.gaps"));
        }
        Ok(())
    }

    /// Initial contract utility, not an analysis engine. Availability must be a prefix.
    pub fn available_len(&self, as_of: i64) -> Result<usize, ContractError> {
        self.validate()?;
        validate_timestamp(as_of, "as_of")?;
        if self.metadata.revision_available_at > as_of {
            return Err(ContractError::new(
                ErrorCode::RevisionNotAvailable,
                "metadata.revision_available_at",
            ));
        }
        Ok(self
            .timestamp
            .iter()
            .zip(&self.available_at)
            .zip(&self.is_closed)
            .take_while(|((timestamp, available), closed)| {
                **timestamp <= as_of && **available <= as_of && **closed
            })
            .count())
    }
}
