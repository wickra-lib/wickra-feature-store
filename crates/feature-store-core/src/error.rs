//! Error type for the feature-store core.

/// Errors produced while parsing a spec, resolving indicators, or building a
/// feature matrix.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A JSON or TOML payload could not be parsed.
    #[error("parse: {0}")]
    Parse(String),
    /// A feature or microstructure metric is not known to the registry.
    #[error("unknown indicator: {0}")]
    UnknownIndicator(String),
    /// The spec is structurally invalid (empty features, zero horizon, ...).
    #[error("bad spec: {0}")]
    BadSpec(String),
    /// Input data was malformed (bad candle, unreadable CSV, ...).
    #[error("data: {0}")]
    Data(String),
    /// An output could not be produced (Arrow/Parquet write failure, ...).
    #[error("output: {0}")]
    Output(String),
}

/// Result alias for the feature-store core.
pub type Result<T> = core::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Parse(e.to_string())
    }
}
