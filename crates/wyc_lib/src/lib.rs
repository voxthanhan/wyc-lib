//! Shared contracts, P1.1 request preparation and P1.2 rolling primitives.
//! Market-level analytical entry points arrive in P1.6–P6.
mod preparation;

pub use contracts;
pub use contracts::{AnalysisRequest, ContractError, MarketData};
pub use indicators;
pub use preparation::{prepare_request, PreparedAnalysis};

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ALGORITHM_VERSION: &str = "p1-v1";
