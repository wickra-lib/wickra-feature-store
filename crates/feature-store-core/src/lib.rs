//! Data-driven feature-matrix core for the Wickra Feature Store.
//!
//! This crate is scaffolded in P-FS-0; the `FeatureSpec` pipeline (indicator
//! set, symbol fold, feature/label evaluation, scaling and matrix output)
//! lands in P-FS-1.

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
