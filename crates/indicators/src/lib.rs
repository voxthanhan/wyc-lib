//! Rolling primitives, P1.3 basic indicators/state and P1.4 indicators.
//! Slice APIs require a causal prefix; calculate_basic/calculate_advanced
//! select market availability themselves.
//! Orchestration for the full P1 catalog arrives in P1.6.
pub mod advanced;
pub mod basic;
mod basic_state;
pub mod incremental;
pub mod rolling;

pub use basic_state::BasicIndicatorState;
