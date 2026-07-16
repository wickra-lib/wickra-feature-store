//! Data-driven feature-matrix core for the Wickra Feature Store.
//!
//! A [`spec::FeatureSpec`] (parsed from JSON or TOML) is folded over each
//! symbol's history: 514 O(1) streaming indicators, raw price fields and
//! microstructure metrics become feature columns, forward-looking targets become
//! label columns, and the result is materialized as a [`matrix::FeatureMatrix`]
//! — deterministic across languages, batch/streaming, and parallel/sequential.
//!
//! The [`FeatureStore`] handle drives everything through one JSON-over-C-ABI
//! entry point, [`FeatureStore::command_json`]; the free [`build`] function is
//! the batch entry. Arrow/Parquet output is native only, behind the `arrow`
//! feature.

pub mod build;
pub mod config;
pub mod error;
pub mod feature;
pub mod feature_store;
pub mod indicator_set;
pub mod label;
pub mod matrix;
pub mod scaling;
pub mod spec;
pub mod symbol_state;
pub mod universe;

#[cfg(feature = "arrow")]
pub mod arrow_out;

pub use build::build;
pub use config::Config;
pub use error::{Error, Result};
pub use feature::{Feature, PriceField};
pub use feature_store::FeatureStore;
pub use indicator_set::IndicatorSet;
pub use label::{forward_return, triple_barrier, Label};
pub use matrix::{round_to, FeatureMatrix, RowId};
pub use scaling::apply_scaling;
pub use spec::{FeatureSpec, OutputFormat, Scaling, WarmupPolicy};
pub use symbol_state::SymbolState;
pub use universe::Universe;

/// The candle input type, re-exported from the shared engine.
pub use wickra_backtest_core::Candle;

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
