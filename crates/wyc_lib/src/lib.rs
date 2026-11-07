//! Deterministic P1 analysis, append/reset state and shared algorithm exports.
mod orchestration;
mod preparation;
mod state;

pub use contracts;
pub use contracts::{
    AnalysisError, AnalysisRequest, AnalysisResult, ClosedBar, ClosedBarUpdate, ContractError,
    MarketData,
};
pub use indicators;
pub use market_structure;
pub use orchestration::{analyze_batch, analyze_structure, calculate_indicators};
pub use preparation::{prepare_request, PreparedAnalysis};
pub use state::AnalysisState;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ALGORITHM_VERSION: &str = "p1-v1";
