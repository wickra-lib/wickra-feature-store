//! The `FeatureSpec` JSON/TOML contract and its structural validation.

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::label::Label;
use serde::{Deserialize, Serialize};

/// The full feature specification: which symbols, which feature and label
/// columns, and how to emit and scale them.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FeatureSpec {
    /// Symbol keys to build (order is irrelevant; the universe iterates sorted).
    pub universe: Vec<String>,
    /// Informational timeframe tag (e.g. `1h`); not interpreted.
    #[serde(default)]
    pub timeframe: Option<String>,
    /// Feature columns, in output column order.
    pub features: Vec<Feature>,
    /// Label columns, emitted after the features.
    #[serde(default)]
    pub labels: Vec<Label>,
    /// Optional trailing window: keep only the last `N` rows per symbol.
    #[serde(default)]
    pub window: Option<usize>,
    /// Output format (JSON by default).
    #[serde(default)]
    pub output: OutputFormat,
    /// Optional per-feature-column scaling.
    #[serde(default)]
    pub scaling: Option<Scaling>,
    /// Warmup row policy.
    #[serde(default)]
    pub warmup: WarmupPolicy,
}

/// Matrix output format.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// Canonical JSON (default).
    #[default]
    Json,
    /// CSV.
    Csv,
    /// Apache Arrow (native only, `arrow` feature).
    Arrow,
    /// Apache Parquet (native only, `arrow` feature).
    Parquet,
}

/// Per-feature-column scaling mode.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scaling {
    /// Z-score: `(x - mean) / std_pop`.
    ZScore,
    /// Min-max: `(x - min) / (max - min)` into `[0, 1]`.
    MinMax,
}

/// Warmup row policy.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WarmupPolicy {
    /// Keep every bar; not-yet-ready cells are `NaN` (default).
    #[default]
    Nan,
    /// Drop a row unless every feature cell is ready.
    Skip,
}

impl FeatureSpec {
    /// Parse a spec from JSON.
    ///
    /// # Errors
    /// Returns [`Error::Parse`] if the JSON is malformed.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// Parse a spec from TOML.
    ///
    /// # Errors
    /// Returns [`Error::Parse`] if the TOML is malformed.
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|e| Error::Parse(e.to_string()))
    }

    /// Structurally validate the spec. The existence of referenced indicators is
    /// checked when the indicator set is built (it resolves against the
    /// registry), so this only covers shape.
    ///
    /// # Errors
    /// Returns [`Error::BadSpec`] on an empty universe or feature list, a zero
    /// label horizon, or a zero window.
    pub fn validate(&self) -> Result<()> {
        if self.universe.is_empty() {
            return Err(Error::BadSpec("universe is empty".into()));
        }
        if self.features.is_empty() {
            return Err(Error::BadSpec("features is empty".into()));
        }
        for label in &self.labels {
            if label.horizon() == 0 {
                return Err(Error::BadSpec("label horizon must be > 0".into()));
            }
        }
        if self.window == Some(0) {
            return Err(Error::BadSpec("window must be > 0".into()));
        }
        Ok(())
    }

    /// The canonical column keys: feature keys (spec order) then label keys.
    #[must_use]
    pub fn columns(&self) -> Vec<String> {
        self.features
            .iter()
            .map(Feature::key)
            .chain(self.labels.iter().map(Label::key))
            .collect()
    }

    /// The number of feature columns (before the labels).
    #[must_use]
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_json() -> &'static str {
        r#"{ "universe": ["AAA","BBB"],
             "features": [
               {"kind":"indicator","name":"rsi","params":[14]},
               {"kind":"price","field":"close"} ],
             "labels": [ {"kind":"forward_return","horizon":5} ] }"#
    }

    #[test]
    fn parses_and_defaults() {
        let s = FeatureSpec::from_json(spec_json()).unwrap();
        assert_eq!(s.output, OutputFormat::Json);
        assert_eq!(s.warmup, WarmupPolicy::Nan);
        assert!(s.scaling.is_none());
        s.validate().unwrap();
    }

    #[test]
    fn columns_are_features_then_labels_in_order() {
        let s = FeatureSpec::from_json(spec_json()).unwrap();
        assert_eq!(s.columns(), vec!["rsi(14)", "price.close", "fwd_return(5)"]);
        assert_eq!(s.feature_count(), 2);
    }

    #[test]
    fn empty_features_rejected() {
        let s = FeatureSpec {
            universe: vec!["AAA".into()],
            timeframe: None,
            features: vec![],
            labels: vec![],
            window: None,
            output: OutputFormat::Json,
            scaling: None,
            warmup: WarmupPolicy::Nan,
        };
        assert!(matches!(s.validate(), Err(Error::BadSpec(_))));
    }

    #[test]
    fn zero_horizon_rejected() {
        let json = r#"{ "universe": ["AAA"],
             "features": [ {"kind":"price","field":"close"} ],
             "labels": [ {"kind":"forward_return","horizon":0} ] }"#;
        let s = FeatureSpec::from_json(json).unwrap();
        assert!(matches!(s.validate(), Err(Error::BadSpec(_))));
    }

    #[test]
    fn toml_parses() {
        let toml = r#"
            universe = ["AAA"]
            [[features]]
            kind = "price"
            field = "close"
        "#;
        let s = FeatureSpec::from_toml(toml).unwrap();
        s.validate().unwrap();
        assert_eq!(s.columns(), vec!["price.close"]);
    }
}
