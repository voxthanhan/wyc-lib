//! Shared contracts, P1.1 preparation and P1.2–P1.5 algorithm crate exports.
//! Market-level analytical entry points arrive in P1.6–P6.
mod preparation;

pub use contracts;
pub use contracts::{AnalysisRequest, ContractError, MarketData};
pub use indicators;
pub use market_structure;
pub use preparation::{prepare_request, PreparedAnalysis};

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ALGORITHM_VERSION: &str = "p1-v1";
