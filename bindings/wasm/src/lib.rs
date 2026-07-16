//! WebAssembly bindings for `wickra-feature-store` (wasm-bindgen).
//!
//! Build feature matrices in the browser: create a `FeatureStore` from a spec
//! JSON, drive it with a command JSON (`set_spec`, `push`, `push_batch`,
//! `build`, `build_batch`, `labels`, `reset`, `version`) and read back the
//! response JSON. The same command protocol crosses every binding, so a browser
//! front-end runs against the exact same core as the native CLI.
//!
//! The fold runs sequentially here (no rayon thread pool in a browser sandbox),
//! which is byte-identical to the native run — the exact cross-language golden
//! check. Columnar (arrow/parquet) output is native-only and not available in
//! the browser.

use wasm_bindgen::prelude::*;

use feature_store_core::FeatureStore as CoreFeatureStore;

/// A feature store driven by JSON commands.
#[wasm_bindgen]
pub struct FeatureStore {
    inner: CoreFeatureStore,
}

#[wasm_bindgen]
impl FeatureStore {
    /// Build a feature store from a spec JSON string.
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<FeatureStore, JsError> {
        CoreFeatureStore::new(spec_json)
            .map(|inner| Self { inner })
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Apply a command JSON (`{"cmd":"...", ...}`) and return the response JSON.
    pub fn command(&mut self, cmd_json: &str) -> Result<String, JsError> {
        self.inner
            .command_json(cmd_json)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// The library version.
    #[wasm_bindgen(js_name = version)]
    pub fn instance_version(&self) -> String {
        CoreFeatureStore::version().to_string()
    }
}

/// The library version.
#[wasm_bindgen]
pub fn version() -> String {
    CoreFeatureStore::version().to_string()
}
