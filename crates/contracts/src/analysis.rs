use crate::{
    AvailableData, ContractError, ErrorCode, FiniteF64, MarketData, CONFIG_VERSION, SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Indicators,
    MarketStructure,
    PriceZones,
    VolumeEngine,
    RelativeStrength,
    WyckoffEngine,
    EvidenceEngine,
    FeatureEngine,
}

impl Module {
    /// Stable wire spelling, also used by the event-v1 identity contract.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Indicators => "indicators",
            Self::MarketStructure => "market_structure",
            Self::PriceZones => "price_zones",
            Self::VolumeEngine => "volume_engine",
            Self::RelativeStrength => "relative_strength",
            Self::WyckoffEngine => "wyckoff_engine",
            Self::EvidenceEngine => "evidence_engine",
            Self::FeatureEngine => "feature_engine",
        }
    }
}

/// All P1 periods are explicit. There are no hidden runtime defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndicatorConfig {
    pub period: u32,
    pub macd_fast: u32,
    pub macd_slow: u32,
    pub macd_signal: u32,
    pub bollinger_multiplier: FiniteF64,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            bollinger_multiplier: crate::value::TWO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureConfig {
    pub left: u32,
    pub right: u32,
}

impl Default for StructureConfig {
    fn default() -> Self {
        Self { left: 2, right: 2 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisConfig {
    pub config_version: String,
    pub indicators: IndicatorConfig,
    pub structure: StructureConfig,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            config_version: CONFIG_VERSION.into(),
            indicators: IndicatorConfig::default(),
            structure: StructureConfig::default(),
        }
    }
}

impl AnalysisConfig {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.config_version != CONFIG_VERSION {
            return Err(ContractError::new(
                ErrorCode::UnsupportedVersion,
                "config.config_version",
            ));
        }
        for (path, period) in [
            ("indicators.period", self.indicators.period),
            ("indicators.macd_fast", self.indicators.macd_fast),
            ("indicators.macd_slow", self.indicators.macd_slow),
            ("indicators.macd_signal", self.indicators.macd_signal),
            ("structure.left", self.structure.left),
            ("structure.right", self.structure.right),
        ] {
            if !(1..=1_000_000).contains(&period) {
                return Err(ContractError::new(
                    ErrorCode::InvalidConfig,
                    format!("config.{path}"),
                ));
            }
        }
        if self.indicators.macd_fast >= self.indicators.macd_slow
            || self.indicators.bollinger_multiplier.get() <= 0.0
        {
            return Err(ContractError::new(
                ErrorCode::InvalidConfig,
                "config.indicators",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRequest {
    pub schema_version: String,
    pub data: MarketData,
    pub as_of: i64,
    pub modules: Vec<Module>,
    pub config: AnalysisConfig,
}

impl AnalysisRequest {
    /// Only P1 module selection is specified for execution. No algorithms run here.
    pub fn validate(&self) -> Result<(), ContractError> {
        self.validate_and_select().map(|_| ())
    }

    /// Validate once and retain a borrowed view of only the available prefix.
    pub fn validate_and_select(&self) -> Result<AvailableData<'_>, ContractError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ContractError::new(
                ErrorCode::UnsupportedVersion,
                "schema_version",
            ));
        }
        let data = AvailableData::new(&self.data, self.as_of)?;
        self.config.validate()?;
        if self.modules.is_empty() {
            return Err(ContractError::new(ErrorCode::InvalidConfig, "modules"));
        }
        let mut seen = BTreeSet::new();
        for module in &self.modules {
            if !seen.insert(module) {
                return Err(ContractError::new(ErrorCode::DuplicateModule, "modules"));
            }
            if !matches!(module, Module::Indicators | Module::MarketStructure) {
                return Err(ContractError::new(ErrorCode::UnsupportedModule, "modules"));
            }
        }
        Ok(data)
    }
}

/// Versioned identity and cursor of a P1 analysis state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateIdentity {
    pub schema_version: String,
    pub engine_version: String,
    pub algorithm_version: String,
    pub symbol: String,
    pub timeframe: String,
    pub data_revision: String,
    pub config_hash: String,
    pub modules: Vec<Module>,
    pub last_timestamp: Option<i64>,
    pub last_as_of: i64,
}

/// One append-only bar. Null numeric fields are reported as missing_data by
/// market validation, just like the columnar batch input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClosedBar {
    pub timestamp: i64,
    pub available_at: i64,
    pub open: Option<FiniteF64>,
    pub high: Option<FiniteF64>,
    pub low: Option<FiniteF64>,
    pub close: Option<FiniteF64>,
    pub volume: Option<FiniteF64>,
    pub is_closed: bool,
}

/// Echo the state's exact identity/cursor and full metadata on every append.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClosedBarUpdate {
    pub identity: StateIdentity,
    pub metadata: crate::MarketMetadata,
    pub bar: ClosedBar,
    pub as_of: i64,
}
