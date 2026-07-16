//! Data-driven feature-matrix core for the Wickra Feature Store.
//!
//! A [`spec::FeatureSpec`] (parsed from JSON or TOML) is folded over each
//! symbol's history: 514 O(1) streaming indicators, raw price fields and
//! microstructure metrics become feature columns, forward-looking targets become
//! label columns, and the result is materialized as a [`matrix::FeatureMatrix`]
//! — deterministic across languages, batch/streaming, and parallel/sequential.
//!
//! The data model (spec, features, labels, matrix, scaling) lands first in
//! P-FS-1; the indicator fold, the parallel symbol build and the JSON-over-C-ABI
//! `command_json` boundary build on top of it.

pub mod error;
pub mod feature;
pub mod label;
pub mod matrix;
pub mod scaling;
pub mod spec;

pub use error::{Error, Result};
pub use feature::{Feature, PriceField};
pub use label::{forward_return, triple_barrier, Label};
pub use matrix::{round_to, FeatureMatrix, RowId};
pub use scaling::apply_scaling;
pub use spec::{FeatureSpec, OutputFormat, Scaling, WarmupPolicy};

/// The crate version, as declared in `Cargo.toml`.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_nonempty() {
        assert!(!super::version().is_empty());
    }
}
