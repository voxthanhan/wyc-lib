use crate::{AnalysisRequest, ContractError, ALGORITHM_VERSION, ENGINE_VERSION};
use contracts::{AnalysisConfig, AvailableData, Module, ResultMetadata, SCHEMA_VERSION};

/// Validated request context, with normalized selection and deterministic
/// metadata. This is preparation, not an analytical result or mutable state.
#[derive(Debug)]
pub struct PreparedAnalysis<'a> {
    data: AvailableData<'a>,
    config: &'a AnalysisConfig,
    modules: Vec<Module>,
    metadata: ResultMetadata,
}

impl<'a> PreparedAnalysis<'a> {
    pub fn data(&self) -> &AvailableData<'a> {
        &self.data
    }

    pub fn config(&self) -> &'a AnalysisConfig {
        self.config
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn metadata(&self) -> &ResultMetadata {
        &self.metadata
    }
}

pub fn prepare_request(request: &AnalysisRequest) -> Result<PreparedAnalysis<'_>, ContractError> {
    let data = request.validate_and_select()?;
    let mut modules = request.modules.clone();
    modules.sort_unstable();
    let metadata = ResultMetadata {
        schema_version: SCHEMA_VERSION.into(),
        engine_version: ENGINE_VERSION.into(),
        algorithm_version: ALGORITHM_VERSION.into(),
        symbol: request.data.symbol.clone(),
        timeframe: request.data.timeframe.clone(),
        data_revision: request.data.metadata.data_revision.clone(),
        config_hash: request.config.config_hash()?,
        as_of: request.as_of,
    };
    Ok(PreparedAnalysis {
        data,
        config: &request.config,
        modules,
        metadata,
    })
}
