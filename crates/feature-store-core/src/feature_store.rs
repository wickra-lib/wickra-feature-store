//! The stateful handle and its single JSON-over-C-ABI entry point.
//!
//! Every binding drives a `FeatureStore` through [`FeatureStore::command_json`],
//! which dispatches the command table in the handoff §6.8 and returns the core's
//! JSON string verbatim — that verbatim return is what makes the output
//! byte-identical across all ten languages. Arrow/Parquet are binary and never
//! cross this boundary; use the CLI or the native [`crate::arrow_out`] API.

use crate::build::build as build_matrix;
use crate::error::{Error, Result};
use crate::indicator_set::IndicatorSet;
use crate::matrix::FeatureMatrix;
use crate::spec::{FeatureSpec, OutputFormat};
use crate::universe::Universe;
use serde::Deserialize;
use std::collections::BTreeMap;
use wickra_backtest_core::Candle;

/// A candle in its JSON input form (`ts` is the timestamp; the core `Candle`
/// calls the field `time`).
#[derive(Deserialize)]
struct CandleInput {
    ts: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl From<CandleInput> for Candle {
    fn from(c: CandleInput) -> Self {
        Candle {
            time: c.ts,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }
}

/// The stateful feature-store handle: a spec plus the pushed candle universe.
pub struct FeatureStore {
    spec: FeatureSpec,
    universe: Universe,
}

impl FeatureStore {
    /// Build a store from a JSON spec, validating shape and indicator names.
    ///
    /// # Errors
    /// Returns [`Error::Parse`] on malformed JSON and [`Error::BadSpec`] /
    /// [`Error::UnknownIndicator`] on an invalid spec.
    pub fn new(spec_json: &str) -> Result<Self> {
        let spec = FeatureSpec::from_json(spec_json)?;
        validate_full(&spec)?;
        Ok(Self {
            spec,
            universe: Universe::new(),
        })
    }

    /// Replace the spec (already validated by the caller).
    pub fn set_spec(&mut self, spec: FeatureSpec) {
        self.spec = spec;
    }

    /// Push one candle onto a symbol's streaming history.
    pub fn push(&mut self, symbol: &str, candle: &Candle) {
        self.universe.push(symbol, *candle);
    }

    /// Build the matrix from the current streaming state.
    ///
    /// # Errors
    /// Propagates [`build_matrix`] errors.
    pub fn build(&self) -> Result<FeatureMatrix> {
        build_matrix(self.universe.data(), &self.spec)
    }

    /// Clear the universe, keeping the spec.
    pub fn reset(&mut self) {
        self.universe.clear();
    }

    /// The crate version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Dispatch one command envelope and return its JSON response.
    ///
    /// # Errors
    /// Returns an [`Error`] for malformed input, an unknown command, an invalid
    /// spec, or a request for a binary format over this boundary. Bindings turn
    /// the error into an `{"ok":false,"error":...}` string.
    pub fn command_json(&mut self, cmd_json: &str) -> Result<String> {
        let envelope: serde_json::Value = serde_json::from_str(cmd_json)?;
        let cmd = envelope
            .get("cmd")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| Error::BadSpec("missing \"cmd\"".into()))?;

        match cmd {
            "set_spec" => {
                let spec: FeatureSpec = parse_field(&envelope, "spec")?;
                validate_full(&spec)?;
                self.set_spec(spec);
                Ok(ok_json())
            }
            "push" => {
                let symbol = str_field(&envelope, "symbol")?;
                let candle: CandleInput = parse_field(&envelope, "candle")?;
                self.push(&symbol, &candle.into());
                Ok(ok_json())
            }
            "push_batch" => {
                let symbol = str_field(&envelope, "symbol")?;
                let candles: Vec<CandleInput> = parse_field(&envelope, "candles")?;
                for candle in candles {
                    self.push(&symbol, &candle.into());
                }
                Ok(ok_json())
            }
            "build" => {
                let format = resolved_format(&envelope, &self.spec)?;
                Ok(render(&self.build()?, format))
            }
            "build_batch" => {
                let data = parse_data(&envelope)?;
                let format = resolved_format(&envelope, &self.spec)?;
                Ok(render(&build_matrix(&data, &self.spec)?, format))
            }
            "labels" => {
                let matrix = self.build()?;
                Ok(labels_only(&matrix, &self.spec).to_json())
            }
            "reset" => {
                self.reset();
                Ok(ok_json())
            }
            "version" => Ok(format!("{{\"version\":\"{}\"}}", Self::version())),
            other => Err(Error::BadSpec(format!("unknown cmd: {other}"))),
        }
    }
}

/// Validate a spec's shape and resolve its indicator names.
fn validate_full(spec: &FeatureSpec) -> Result<()> {
    spec.validate()?;
    IndicatorSet::from_features(&spec.features)?;
    Ok(())
}

fn ok_json() -> String {
    "{\"ok\":true}".to_string()
}

fn str_field(envelope: &serde_json::Value, key: &str) -> Result<String> {
    envelope
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| Error::BadSpec(format!("missing string field \"{key}\"")))
}

fn parse_field<T: serde::de::DeserializeOwned>(
    envelope: &serde_json::Value,
    key: &str,
) -> Result<T> {
    let value = envelope
        .get(key)
        .ok_or_else(|| Error::BadSpec(format!("missing field \"{key}\"")))?;
    serde_json::from_value(value.clone()).map_err(|e| Error::Parse(e.to_string()))
}

fn parse_data(envelope: &serde_json::Value) -> Result<BTreeMap<String, Vec<Candle>>> {
    let raw: BTreeMap<String, Vec<CandleInput>> = parse_field(envelope, "data")?;
    Ok(raw
        .into_iter()
        .map(|(symbol, candles)| (symbol, candles.into_iter().map(Into::into).collect()))
        .collect())
}

/// The output format for a `build`/`build_batch` command: the request override,
/// else the spec default. Binary formats are rejected on this boundary.
fn resolved_format(envelope: &serde_json::Value, spec: &FeatureSpec) -> Result<OutputFormat> {
    let format = match envelope.get("format").and_then(serde_json::Value::as_str) {
        Some("json") => OutputFormat::Json,
        Some("csv") => OutputFormat::Csv,
        Some(other) => {
            return Err(Error::Output(format!(
                "unsupported format over FFI: {other}"
            )))
        }
        None => spec.output,
    };
    match format {
        OutputFormat::Json | OutputFormat::Csv => Ok(format),
        OutputFormat::Arrow | OutputFormat::Parquet => {
            Err(Error::Output("use CLI/native API for parquet".into()))
        }
    }
}

fn render(matrix: &FeatureMatrix, format: OutputFormat) -> String {
    match format {
        OutputFormat::Csv => matrix.to_csv(),
        _ => matrix.to_json(),
    }
}

/// A view of only the label columns, with the same rows and index.
fn labels_only(matrix: &FeatureMatrix, spec: &FeatureSpec) -> FeatureMatrix {
    let start = spec.feature_count();
    let columns: Vec<String> = matrix.columns[start..].to_vec();
    let mut out = FeatureMatrix::new(columns);
    for (id, row) in matrix.index.iter().zip(&matrix.data) {
        out.push_row(id.clone(), row[start..].to_vec());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = r#"{ "universe": ["AAA"],
        "features": [ {"kind":"price","field":"close"} ],
        "labels": [ {"kind":"forward_return","horizon":1} ] }"#;

    fn push_series(fs: &mut FeatureStore, closes: &[f64]) {
        for (i, &c) in closes.iter().enumerate() {
            let cmd = format!(
                r#"{{"cmd":"push","symbol":"AAA","candle":{{"ts":{i},"open":{c},"high":{c},"low":{c},"close":{c},"volume":1}}}}"#
            );
            fs.command_json(&cmd).unwrap();
        }
    }

    #[test]
    fn version_command() {
        let mut fs = FeatureStore::new(SPEC).unwrap();
        assert_eq!(
            fs.command_json(r#"{"cmd":"version"}"#).unwrap(),
            format!("{{\"version\":\"{}\"}}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn streaming_build_matches_batch() {
        let mut fs = FeatureStore::new(SPEC).unwrap();
        push_series(&mut fs, &[100.0, 110.0, 121.0]);
        let streamed = fs.command_json(r#"{"cmd":"build"}"#).unwrap();

        let batch = fs
            .command_json(
                r#"{"cmd":"build_batch","data":{"AAA":[
                    {"ts":0,"open":100,"high":100,"low":100,"close":100,"volume":1},
                    {"ts":1,"open":110,"high":110,"low":110,"close":110,"volume":1},
                    {"ts":2,"open":121,"high":121,"low":121,"close":121,"volume":1}]}}"#,
            )
            .unwrap();
        assert_eq!(streamed, batch);
        assert!(streamed.contains("\"price.close\""));
    }

    #[test]
    fn labels_command_projects_label_columns() {
        let mut fs = FeatureStore::new(SPEC).unwrap();
        push_series(&mut fs, &[100.0, 110.0]);
        let labels = fs.command_json(r#"{"cmd":"labels"}"#).unwrap();
        assert!(labels.contains("fwd_return(1)"));
        assert!(!labels.contains("price.close"));
    }

    #[test]
    fn reset_clears_universe() {
        let mut fs = FeatureStore::new(SPEC).unwrap();
        push_series(&mut fs, &[100.0, 110.0]);
        assert_eq!(fs.command_json(r#"{"cmd":"reset"}"#).unwrap(), ok_json());
        let built = fs.command_json(r#"{"cmd":"build"}"#).unwrap();
        assert!(built.contains("\"rows\":0"));
    }

    #[test]
    fn csv_format_override() {
        let mut fs = FeatureStore::new(SPEC).unwrap();
        push_series(&mut fs, &[100.0, 110.0]);
        let csv = fs
            .command_json(r#"{"cmd":"build","format":"csv"}"#)
            .unwrap();
        assert!(csv.starts_with("symbol,ts,price.close,fwd_return(1)"));
    }

    #[test]
    fn unknown_cmd_errors() {
        let mut fs = FeatureStore::new(SPEC).unwrap();
        assert!(fs.command_json(r#"{"cmd":"frobnicate"}"#).is_err());
    }

    #[test]
    fn parquet_over_ffi_is_rejected() {
        let spec = r#"{ "universe": ["AAA"], "output":"parquet",
            "features": [ {"kind":"price","field":"close"} ] }"#;
        let mut fs = FeatureStore::new(spec).unwrap();
        assert!(fs.command_json(r#"{"cmd":"build"}"#).is_err());
    }

    #[test]
    fn bad_spec_over_command_errors() {
        let mut fs = FeatureStore::new(SPEC).unwrap();
        let bad = r#"{"cmd":"set_spec","spec":{"universe":[],"features":[]}}"#;
        assert!(fs.command_json(bad).is_err());
    }
}
