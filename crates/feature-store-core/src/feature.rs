//! Feature column definitions and their canonical string keys.

use serde::{Deserialize, Serialize};

/// One feature column: an indicator, a raw price field, or a microstructure
/// metric. The column order in a matrix is the order these appear in the spec.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Feature {
    /// A streaming indicator resolved by name and parameters, optionally a named
    /// sub-output field of a multi-output indicator.
    Indicator {
        /// Registry indicator name (e.g. `rsi`, `macd`).
        name: String,
        /// Indicator parameters (e.g. `[14]`, `[12, 26, 9]`).
        #[serde(default)]
        params: Vec<f64>,
        /// Named sub-output field, or the primary value when absent.
        #[serde(default)]
        field: Option<String>,
    },
    /// A raw OHLCV field of the bar.
    Price {
        /// Which price field to read.
        field: PriceField,
    },
    /// A microstructure metric (orderflow, funding, open interest, liquidations),
    /// resolved through the same registry as [`Feature::Indicator`].
    Microstructure {
        /// Registry metric name (e.g. `ofi`).
        metric: String,
        /// Metric parameters.
        #[serde(default)]
        params: Vec<f64>,
    },
}

/// A raw price field of a candle.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriceField {
    /// Opening price.
    Open,
    /// Highest price.
    High,
    /// Lowest price.
    Low,
    /// Closing price.
    Close,
    /// Traded volume.
    Volume,
}

impl PriceField {
    /// The lowercase field name used in a column key.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            PriceField::Open => "open",
            PriceField::High => "high",
            PriceField::Low => "low",
            PriceField::Close => "close",
            PriceField::Volume => "volume",
        }
    }
}

impl Feature {
    /// The canonical column key:
    /// `rsi(14)` / `macd(12,26,9).hist` for indicators, `price.close` for price
    /// fields, `ms.ofi(20)` for microstructure metrics. The formatting is fixed
    /// so keys are byte-identical across languages.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Feature::Indicator {
                name,
                params,
                field,
            } => {
                let base = format!("{name}({})", fmt_params(params));
                match field {
                    Some(f) => format!("{base}.{f}"),
                    None => base,
                }
            }
            Feature::Price { field } => format!("price.{}", field.as_str()),
            Feature::Microstructure { metric, params } => {
                format!("ms.{metric}({})", fmt_params(params))
            }
        }
    }
}

/// Format a parameter list for a key: whole values as integers, otherwise the
/// default float rendering, comma-joined.
pub(crate) fn fmt_params(params: &[f64]) -> String {
    params
        .iter()
        .map(|p| fmt_num(*p))
        .collect::<Vec<_>>()
        .join(",")
}

/// Format a single number for a key: whole values as integers, else default
/// float rendering.
pub(crate) fn fmt_num(v: f64) -> String {
    if v.is_finite() && v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_key() {
        let f = Feature::Indicator {
            name: "rsi".into(),
            params: vec![14.0],
            field: None,
        };
        assert_eq!(f.key(), "rsi(14)");
    }

    #[test]
    fn indicator_field_key() {
        let f = Feature::Indicator {
            name: "macd".into(),
            params: vec![12.0, 26.0, 9.0],
            field: Some("hist".into()),
        };
        assert_eq!(f.key(), "macd(12,26,9).hist");
    }

    #[test]
    fn price_key() {
        let f = Feature::Price {
            field: PriceField::Close,
        };
        assert_eq!(f.key(), "price.close");
    }

    #[test]
    fn microstructure_key() {
        let f = Feature::Microstructure {
            metric: "ofi".into(),
            params: vec![20.0],
        };
        assert_eq!(f.key(), "ms.ofi(20)");
    }

    #[test]
    fn fractional_param_key() {
        let f = Feature::Indicator {
            name: "kama".into(),
            params: vec![10.0, 2.5],
            field: None,
        };
        assert_eq!(f.key(), "kama(10,2.5)");
    }

    #[test]
    fn feature_json_roundtrip() {
        let json = r#"{"kind":"indicator","name":"macd","params":[12,26,9],"field":"hist"}"#;
        let f: Feature = serde_json::from_str(json).unwrap();
        assert_eq!(f.key(), "macd(12,26,9).hist");
    }
}
