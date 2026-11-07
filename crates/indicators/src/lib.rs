//! Rolling primitives and P1.3 basic indicators/state. Slice APIs require a
//! causal prefix; basic::calculate_basic selects market availability itself.
//! Orchestration for the full P1 catalog arrives in P1.6.
pub mod basic;
mod basic_state;
pub mod rolling;

pub use basic_state::BasicIndicatorState;
