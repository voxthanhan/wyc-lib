use crate::{analyze_batch, prepare_request, AnalysisError, AnalysisRequest, AnalysisResult};
use contracts::{
    validate_timestamp, AnalysisConfig, AvailableBar, ClosedBarUpdate, ContractError, ErrorCode,
    MarketMetadata, Module, StateIdentity,
};
use indicators::incremental::IndicatorMemory;
use market_structure::StructureMemory;

/// Incremental P1 cache. Computation appends only new samples/events; returning
/// the owned complete report still costs O(N). No serialized-state ABI.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisState {
    config: AnalysisConfig,
    metadata: MarketMetadata,
    bars: Vec<AvailableBar>,
    identity: StateIdentity,
    result: AnalysisResult,
    indicators: Option<IndicatorMemory>,
    structure: Option<StructureMemory>,
}
impl AnalysisState {
    pub fn new(request: &AnalysisRequest) -> Result<Self, AnalysisError> {
        let prepared = prepare_request(request)?;
        // Initialization/reset retain canonical batch validation/error priority.
        let result = analyze_batch(request)?;
        let bars = (0..prepared.data().len())
            .map(|i| {
                prepared
                    .data()
                    .bar(i)
                    .ok_or_else(|| error(ErrorCode::InvalidResult, "state.history"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut indicators = if prepared.modules().contains(&Module::Indicators) {
            Some(IndicatorMemory::new(&request.config)?)
        } else {
            None
        };
        let mut structure = if prepared.modules().contains(&Module::MarketStructure) {
            Some(StructureMemory::default())
        } else {
            None
        };
        for i in 0..bars.len() {
            if let Some(memory) = &indicators {
                indicators = Some(memory.prepare(&bars[..=i])?.0);
            }
            if let Some(memory) = &structure {
                structure = Some(
                    memory
                        .prepare(&bars[..=i], &request.config.structure, &result.metadata)?
                        .0,
                );
            }
        }
        let m = &result.metadata;
        let identity = StateIdentity {
            schema_version: m.schema_version.clone(),
            engine_version: m.engine_version.clone(),
            algorithm_version: m.algorithm_version.clone(),
            symbol: m.symbol.clone(),
            timeframe: m.timeframe.clone(),
            data_revision: m.data_revision.clone(),
            config_hash: m.config_hash.clone(),
            modules: prepared.modules().to_vec(),
            last_timestamp: bars.last().map(|b| b.timestamp),
            last_as_of: request.as_of,
        };
        Ok(Self {
            config: request.config.clone(),
            metadata: request.data.metadata.clone(),
            bars,
            identity,
            result,
            indicators,
            structure,
        })
    }
    pub fn result(&self) -> &AnalysisResult {
        &self.result
    }
    pub fn identity(&self) -> &StateIdentity {
        &self.identity
    }
    pub fn history_len(&self) -> usize {
        self.bars.len()
    }

    /// Stage bounded caches and deltas; errors preserve all semantic state.
    pub fn update_closed_bar(
        &mut self,
        update: &ClosedBarUpdate,
    ) -> Result<AnalysisResult, AnalysisError> {
        if !update.metadata.gaps.is_empty() {
            return Err(error(ErrorCode::ResetRequired, "metadata.gaps"));
        }
        if update.identity != self.identity || update.metadata != self.metadata {
            return Err(error(ErrorCode::StateMismatch, "state.identity"));
        }
        validate_timestamp(update.as_of, "as_of")?;
        if update.as_of < self.identity.last_as_of {
            return Err(error(ErrorCode::InvalidAvailability, "as_of"));
        }
        if !update.bar.is_closed {
            return Err(error(ErrorCode::InvalidAvailability, "state.new_bar"));
        }
        let bar = AvailableBar::from_closed(
            &update.bar,
            self.bars.last(),
            self.bars.len(),
            self.metadata.revision_available_at,
            update.as_of,
        )?;
        let mut metadata = self.result.metadata.clone();
        metadata.as_of = update.as_of;
        self.bars.push(bar);
        let staged = (|| {
            let indicators = self
                .indicators
                .as_ref()
                .map(|s| s.prepare(&self.bars))
                .transpose()?;
            let structure = self
                .structure
                .as_ref()
                .map(|s| s.prepare(&self.bars, &self.config.structure, &metadata))
                .transpose()?;
            Ok::<_, AnalysisError>((indicators, structure))
        })();
        let (indicators, structure) = match staged {
            Ok(value) => value,
            Err(e) => {
                self.bars.pop();
                // Only a failed stage may replay batch, to retain historical
                // error priority when multiple arithmetic operations fail.
                let mut request = self.snapshot(update.as_of);
                request.data.timestamp.push(update.bar.timestamp);
                request.data.available_at.push(update.bar.available_at);
                request.data.open.push(update.bar.open);
                request.data.high.push(update.bar.high);
                request.data.low.push(update.bar.low);
                request.data.close.push(update.bar.close);
                request.data.volume.push(update.bar.volume);
                request.data.is_closed.push(update.bar.is_closed);
                return Err(analyze_batch(&request).err().unwrap_or(e));
            }
        };
        if let Some((memory, delta)) = indicators {
            if let Some(output) = &mut self.result.indicators {
                for (kind, sample) in delta {
                    output.series.entry(kind).or_default().push(sample);
                }
            }
            self.indicators = Some(memory);
        }
        if let Some((memory, delta)) = structure {
            if let Some(output) = &mut self.result.structure {
                market_structure::append_delta(output, delta);
            }
            self.structure = Some(memory);
        }
        self.identity.last_timestamp = Some(bar.timestamp);
        self.identity.last_as_of = update.as_of;
        self.result.metadata = metadata;
        Ok(self.result.clone())
    }
    pub fn reset(&mut self, request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError> {
        let replacement = Self::new(request)?;
        let result = replacement.result.clone();
        *self = replacement;
        Ok(result)
    }
    fn snapshot(&self, as_of: i64) -> AnalysisRequest {
        AnalysisRequest {
            schema_version: self.identity.schema_version.clone(),
            as_of,
            modules: self.identity.modules.clone(),
            config: self.config.clone(),
            data: crate::MarketData {
                schema_version: self.identity.schema_version.clone(),
                symbol: self.identity.symbol.clone(),
                timeframe: self.identity.timeframe.clone(),
                metadata: self.metadata.clone(),
                timestamp: self.bars.iter().map(|b| b.timestamp).collect(),
                available_at: self.bars.iter().map(|b| b.available_at).collect(),
                open: self.bars.iter().map(|b| Some(b.open)).collect(),
                high: self.bars.iter().map(|b| Some(b.high)).collect(),
                low: self.bars.iter().map(|b| Some(b.low)).collect(),
                close: self.bars.iter().map(|b| Some(b.close)).collect(),
                volume: self.bars.iter().map(|b| Some(b.volume)).collect(),
                is_closed: vec![true; self.bars.len()],
            },
        }
    }
}
fn error(code: ErrorCode, path: &str) -> AnalysisError {
    ContractError::new(code, path).into()
}
