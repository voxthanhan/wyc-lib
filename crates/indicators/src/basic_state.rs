use crate::basic::{calculate_basic, BasicIndicatorResult};
use contracts::{
    AnalysisConfig, AnalysisError, AvailableData, ContractError, ErrorCode, MarketData,
    ModuleResult,
};

/// Reference state for P1.3 only. Updates consume a cumulative snapshot and
/// recompute batch; no O(1) update or serialized-state compatibility is claimed.
/// Retained data is restricted to the already available prefix.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicIndicatorState {
    data: MarketData,
    config: AnalysisConfig,
    result: ModuleResult<BasicIndicatorResult>,
}

impl BasicIndicatorState {
    pub fn new(
        data: &MarketData,
        config: &AnalysisConfig,
        as_of: i64,
    ) -> Result<Self, AnalysisError> {
        let result = calculate_basic(data, config, as_of)?;
        let len = result.output.series.values().next().map_or(0, Vec::len);
        Ok(Self {
            data: prefix(data, len),
            config: config.clone(),
            result,
        })
    }

    pub fn result(&self) -> &ModuleResult<BasicIndicatorResult> {
        &self.result
    }

    /// Accept exactly one additional closed, available bar without changing the
    /// prior history or identity. A failed update leaves the entire state intact.
    pub fn update_closed_bar(
        &mut self,
        data: &MarketData,
        config: &AnalysisConfig,
        as_of: i64,
    ) -> Result<ModuleResult<BasicIndicatorResult>, AnalysisError> {
        if !data.metadata.gaps.is_empty() {
            return Err(error(ErrorCode::ResetRequired, "metadata.gaps"));
        }
        let available = AvailableData::new(data, as_of)?;
        config.validate()?;
        if data.schema_version != self.data.schema_version
            || data.symbol != self.data.symbol
            || data.timeframe != self.data.timeframe
            || data.metadata != self.data.metadata
            || *config != self.config
        {
            return Err(error(ErrorCode::StateMismatch, "state.identity"));
        }
        if as_of < self.result.metadata.as_of {
            return Err(error(ErrorCode::InvalidAvailability, "as_of"));
        }
        let old_len = self.data.timestamp.len();
        if available.len() < old_len || prefix(data, old_len) != self.data {
            return Err(error(ErrorCode::ResetRequired, "state.history"));
        }
        if available.len() == old_len {
            return Err(error(ErrorCode::InvalidAvailability, "state.new_bar"));
        }
        if available.len() - old_len != 1 {
            return Err(error(ErrorCode::ResetRequired, "state.new_bar"));
        }
        let replacement = Self::new(data, config, as_of)?;
        let result = replacement.result.clone();
        *self = replacement;
        Ok(result)
    }

    /// Rebuild on a new snapshot/config/revision, replacing state only on success.
    pub fn reset(
        &mut self,
        data: &MarketData,
        config: &AnalysisConfig,
        as_of: i64,
    ) -> Result<ModuleResult<BasicIndicatorResult>, AnalysisError> {
        let replacement = Self::new(data, config, as_of)?;
        let result = replacement.result.clone();
        *self = replacement;
        Ok(result)
    }
}

fn error(code: ErrorCode, path: &str) -> AnalysisError {
    ContractError::new(code, path).into()
}

// Copy only the selected prefix; do not clone or retain unavailable OHLCV.
fn prefix(data: &MarketData, len: usize) -> MarketData {
    MarketData {
        schema_version: data.schema_version.clone(),
        symbol: data.symbol.clone(),
        timeframe: data.timeframe.clone(),
        timestamp: data.timestamp[..len].to_vec(),
        available_at: data.available_at[..len].to_vec(),
        open: data.open[..len].to_vec(),
        high: data.high[..len].to_vec(),
        low: data.low[..len].to_vec(),
        close: data.close[..len].to_vec(),
        volume: data.volume[..len].to_vec(),
        is_closed: data.is_closed[..len].to_vec(),
        metadata: data.metadata.clone(),
    }
}
