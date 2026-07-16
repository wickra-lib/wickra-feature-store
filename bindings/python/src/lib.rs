//! Python bindings for `wickra-feature-store`, exposed under the
//! `wickra_feature_store` package.
//!
//! Thin glue over the feature-store core's data-driven surface: build a
//! [`FeatureStore`] from a spec JSON, drive it with a command JSON and read back
//! the response JSON. The same command protocol crosses every binding, so a
//! Python front-end drives the exact same core as the native CLI.

// PyO3 protocol methods take `self` by value/ref regardless of use.
#![allow(clippy::needless_pass_by_value)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use feature_store_core::FeatureStore;

/// A feature store driven by JSON commands.
///
/// `unsendable`: the store holds a streaming universe of stateful indicator
/// evaluators, so a handle is bound to the thread that created it.
#[pyclass(name = "FeatureStore", unsendable)]
struct PyFeatureStore {
    inner: FeatureStore,
}

#[pymethods]
impl PyFeatureStore {
    /// Build a feature store from a spec JSON string.
    #[new]
    fn new(spec_json: &str) -> PyResult<Self> {
        FeatureStore::new(spec_json)
            .map(|inner| Self { inner })
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Apply a command JSON and return the resulting response JSON.
    fn command(&mut self, cmd_json: &str) -> PyResult<String> {
        self.inner
            .command_json(cmd_json)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// The library version.
    #[staticmethod]
    fn version() -> &'static str {
        FeatureStore::version()
    }
}

/// The native module (`wickra_feature_store._wickra_feature_store`).
#[pymodule]
fn _wickra_feature_store(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyFeatureStore>()?;
    Ok(())
}
