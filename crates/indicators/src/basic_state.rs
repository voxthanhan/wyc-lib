use crate::basic::{calculate_basic, BasicIndicatorResult};
use crate::incremental::IndicatorMemory;
use contracts::{
    AnalysisConfig, AnalysisError, AvailableBar, AvailableData, ContractError, ErrorCode,
    MarketData, ModuleResult,
};

/// Reference state for P1.3 only. Updates consume a cumulative snapshot and
/// calculate only new samples. Snapshot validation/history comparison and full
/// owned output remain linear; no O(1) total update or serialized ABI is claimed.
/// Retained data is restricted to the already available prefix.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicIndicatorState {
    data: MarketData,
    config: AnalysisConfig,
    result: ModuleResult<BasicIndicatorResult>,
    memory: IndicatorMemory,
    bars: Vec<AvailableBar>,
}

impl BasicIndicatorState {
    pub fn new(
        data: &MarketData,
        config: &AnalysisConfig,
        as_of: i64,
    ) -> Result<Self, AnalysisError> {
        let result = calculate_basic(data, config, as_of)?;
        let len = result.output.series.values().next().map_or(0, Vec::len);
        let available = AvailableData::new(data, as_of)?;
        let bars = (0..len)
            .map(|i| {
                available
                    .bar(i)
                    .ok_or_else(|| error(ErrorCode::InvalidResult, "state.history"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut memory = IndicatorMemory::new_basic(config)?;
        for i in 0..len {
            memory = memory.prepare(&bars[..=i])?.0;
        }
        Ok(Self {
            data: prefix(data, len),
            config: config.clone(),
            result,
            memory,
            bars,
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
        if available.len() < old_len || !same_prefix(data, &self.data) {
            return Err(error(ErrorCode::ResetRequired, "state.history"));
        }
        if available.len() == old_len {
            return Err(error(ErrorCode::InvalidAvailability, "state.new_bar"));
        }
        if available.len() - old_len != 1 {
            return Err(error(ErrorCode::ResetRequired, "state.new_bar"));
        }
        let bar = available
            .bar(old_len)
            .ok_or_else(|| error(ErrorCode::InvalidResult, "state.new_bar"))?;
        self.bars.push(bar);
        let staged = self.memory.prepare(&self.bars);
        let (memory, delta) = match staged {
            Ok(value) => value,
            Err(e) => {
                self.bars.pop();
                return Err(calculate_basic(data, config, as_of).err().unwrap_or(e));
            }
        };
        self.data.timestamp.push(bar.timestamp);
        self.data.available_at.push(bar.available_at);
        self.data.open.push(Some(bar.open));
        self.data.high.push(Some(bar.high));
        self.data.low.push(Some(bar.low));
        self.data.close.push(Some(bar.close));
        self.data.volume.push(Some(bar.volume));
        self.data.is_closed.push(true);
        for (kind, sample) in delta {
            self.result
                .output
                .series
                .entry(kind)
                .or_default()
                .push(sample);
        }
        self.memory = memory;
        self.result.metadata.as_of = as_of;
        Ok(self.result.clone())
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

fn same_prefix(data: &MarketData, old: &MarketData) -> bool {
    let n = old.timestamp.len();
    data.timestamp[..n] == old.timestamp
        && data.available_at[..n] == old.available_at
        && data.is_closed[..n] == old.is_closed
        && data.open[..n] == old.open
        && data.high[..n] == old.high
        && data.low[..n] == old.low
        && data.close[..n] == old.close
        && data.volume[..n] == old.volume
}
