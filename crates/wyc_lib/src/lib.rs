//! Shared analytical contracts. Analytical entry points arrive in P1–P6;
//! P0 deliberately exposes no placeholder analysis functions.
pub use contracts;
pub use contracts::{AnalysisRequest, ContractError, MarketData};

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
