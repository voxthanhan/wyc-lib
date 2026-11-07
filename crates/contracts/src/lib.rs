//! Versioned wire types and contract checks, independent of algorithms and I/O.
mod analysis;
mod available;
mod error;
mod identity;
mod market;
mod report;
mod result;
mod value;

pub use analysis::*;
pub use available::*;
pub use error::*;
pub use identity::*;
pub use market::*;
pub use report::*;
pub use result::*;
pub use value::*;

pub const SCHEMA_VERSION: &str = "0.1.0";
pub const CONFIG_VERSION: &str = "p1-v1";
