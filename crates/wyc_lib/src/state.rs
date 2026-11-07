use crate::{analyze_batch, prepare_request, AnalysisError, AnalysisRequest, AnalysisResult};
use contracts::{validate_timestamp, ClosedBarUpdate, ContractError, ErrorCode, StateIdentity};

/// Reference P1 state: retained available history and full batch recomputation.
/// No serialized-state compatibility or constant-time update is promised.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisState {
    request: AnalysisRequest,
    identity: StateIdentity,
    result: AnalysisResult,
}

impl AnalysisState {
    pub fn new(request: &AnalysisRequest) -> Result<Self, AnalysisError> {
        let prepared = prepare_request(request)?;
        let result = analyze_batch(request)?;
        let len = prepared.data().len();
        let data = &request.data;
        // Copy only the available prefix; do not clone future values first.
        let retained = crate::MarketData {
            schema_version: data.schema_version.clone(),
            symbol: data.symbol.clone(),
            timeframe: data.timeframe.clone(),
            metadata: data.metadata.clone(),
            timestamp: data.timestamp[..len].to_vec(),
            available_at: data.available_at[..len].to_vec(),
            open: data.open[..len].to_vec(),
            high: data.high[..len].to_vec(),
            low: data.low[..len].to_vec(),
            close: data.close[..len].to_vec(),
            volume: data.volume[..len].to_vec(),
            is_closed: data.is_closed[..len].to_vec(),
        };
        let metadata = &result.metadata;
        let identity = StateIdentity {
            schema_version: metadata.schema_version.clone(),
            engine_version: metadata.engine_version.clone(),
            algorithm_version: metadata.algorithm_version.clone(),
            symbol: metadata.symbol.clone(),
            timeframe: metadata.timeframe.clone(),
            data_revision: metadata.data_revision.clone(),
            config_hash: metadata.config_hash.clone(),
            modules: prepared.modules().to_vec(),
            last_timestamp: retained.timestamp.last().copied(),
            last_as_of: request.as_of,
        };
        Ok(Self {
            request: AnalysisRequest {
                schema_version: request.schema_version.clone(),
                data: retained,
                as_of: request.as_of,
                config: request.config.clone(),
                modules: identity.modules.clone(),
            },
            identity,
            result,
        })
    }

    pub fn result(&self) -> &AnalysisResult {
        &self.result
    }
    pub fn identity(&self) -> &StateIdentity {
        &self.identity
    }
    pub fn history_len(&self) -> usize {
        self.request.data.timestamp.len()
    }

    /// Append exactly one closed, available bar. Errors leave all state intact.
    pub fn update_closed_bar(
        &mut self,
        update: &ClosedBarUpdate,
    ) -> Result<AnalysisResult, AnalysisError> {
        if !update.metadata.gaps.is_empty() {
            return Err(error(ErrorCode::ResetRequired, "metadata.gaps"));
        }
        if update.identity != self.identity || update.metadata != self.request.data.metadata {
            return Err(error(ErrorCode::StateMismatch, "state.identity"));
        }
        validate_timestamp(update.as_of, "as_of")?;
        if update.as_of < self.identity.last_as_of {
            return Err(error(ErrorCode::InvalidAvailability, "as_of"));
        }
        if !update.bar.is_closed {
            return Err(error(ErrorCode::InvalidAvailability, "state.new_bar"));
        }
        let mut request = self.request.clone();
        request.as_of = update.as_of;
        let data = &mut request.data;
        let bar = &update.bar;
        data.timestamp.push(bar.timestamp);
        data.available_at.push(bar.available_at);
        data.open.push(bar.open);
        data.high.push(bar.high);
        data.low.push(bar.low);
        data.close.push(bar.close);
        data.volume.push(bar.volume);
        data.is_closed.push(bar.is_closed);
        let available = request.validate_and_select()?;
        if available.len() != self.history_len() + 1 {
            return Err(error(ErrorCode::InvalidAvailability, "state.new_bar"));
        }
        let replacement = Self::new(&request)?;
        let result = replacement.result.clone();
        *self = replacement;
        Ok(result)
    }

    /// Reset/replay a new snapshot, identity or revision; replace only on success.
    pub fn reset(&mut self, request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError> {
        let replacement = Self::new(request)?;
        let result = replacement.result.clone();
        *self = replacement;
        Ok(result)
    }
}

fn error(code: ErrorCode, path: &str) -> AnalysisError {
    ContractError::new(code, path).into()
}
