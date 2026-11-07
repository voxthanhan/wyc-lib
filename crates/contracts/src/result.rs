use crate::{Evidence, FiniteF64, NumericSample, NumericValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultMetadata {
    pub schema_version: String,
    pub engine_version: String,
    pub algorithm_version: String,
    pub symbol: String,
    pub timeframe: String,
    pub data_revision: String,
    pub config_hash: String,
    pub as_of: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleResult<T> {
    pub metadata: ResultMetadata,
    pub output: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorKind {
    Sum,
    Sma,
    Ema,
    Wma,
    Min,
    Max,
    Variance,
    Std,
    Tr,
    Atr,
    Momentum,
    Roc,
    VolumeSma,
    Rvol,
    Rsi,
    MacdLine,
    MacdSignal,
    MacdHistogram,
    PlusDi,
    MinusDi,
    Dx,
    Adx,
    BollingerMiddle,
    BollingerUpper,
    BollingerLower,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndicatorResult {
    pub series: BTreeMap<IndicatorKind, Vec<NumericSample>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PivotKind {
    High,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PivotLabel {
    Unclassified,
    Hh,
    Hl,
    Lh,
    Ll,
    Eh,
    El,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pivot {
    pub pivot_id: String,
    pub kind: PivotKind,
    pub label: PivotLabel,
    pub level: FiniteF64,
    pub observed_at: i64,
    pub known_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trend {
    Unknown,
    Uptrend,
    Downtrend,
    Sideways,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrendSample {
    pub observed_at: i64,
    pub known_at: i64,
    pub trend: Trend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakKind {
    Bos,
    Choch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureBreak {
    pub event_id: String,
    pub reference_pivot_id: String,
    pub kind: BreakKind,
    pub direction: Direction,
    pub level: FiniteF64,
    pub close: FiniteF64,
    pub strength: NumericValue,
    pub observed_at: i64,
    pub known_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureResult {
    pub pivots: Vec<Pivot>,
    pub trends: Vec<TrendSample>,
    pub breaks: Vec<StructureBreak>,
    pub evidence: Vec<Evidence>,
}

/// Typed output of the P1 batch and state entry points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisResult {
    pub metadata: ResultMetadata,
    pub indicators: Option<IndicatorResult>,
    pub structure: Option<StructureResult>,
}
