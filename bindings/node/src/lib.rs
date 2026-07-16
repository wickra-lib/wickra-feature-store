//! Node.js bindings for `wickra-feature-store` via napi-rs.
//!
//! A `FeatureStore` is built from a spec JSON; `command` takes a request JSON and
//! returns the response JSON, so Node drives the exact same byte-identical surface
//! as every other binding.

use napi_derive::napi;

/// A feature store driven by JSON commands.
#[napi]
pub struct FeatureStore(feature_store_core::FeatureStore);

#[napi]
impl FeatureStore {
    /// Build a feature store from a spec JSON string.
    #[napi(constructor)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(spec_json: String) -> napi::Result<Self> {
        feature_store_core::FeatureStore::new(&spec_json)
            .map(FeatureStore)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Apply a command envelope (`{"cmd":"...", ...}`) and return the response JSON.
    #[napi]
    #[allow(clippy::needless_pass_by_value)]
    pub fn command(&mut self, cmd_json: String) -> napi::Result<String> {
        self.0
            .command_json(&cmd_json)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// The crate version.
    #[napi]
    pub fn version(&self) -> &'static str {
        feature_store_core::FeatureStore::version()
    }
}
