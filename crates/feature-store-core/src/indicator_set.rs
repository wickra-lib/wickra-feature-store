//! Resolves the indicators a spec references and folds candles through them.
//!
//! Indicators are resolved by name and parameters from the `wickra-core`
//! registry, reused through the `wickra-backtest-core` factory — the only
//! name -> indicator resolver in the ecosystem. Each resolved indicator is an
//! object-safe `EvalIndicator`, driven with a candle-only [`BarInput`] (no
//! reference series, derivatives, order book or trades). Microstructure metrics
//! resolve through the very same registry, only from the microstructure
//! namespace, so they share this code path.

use crate::error::{Error, Result};
use crate::feature::{fmt_params, Feature};
use std::collections::BTreeMap;
use wickra_backtest_core::registry::{build, BarInput};
use wickra_backtest_core::{Candle, EvalIndicator};

/// One resolved indicator plus its canonical registry key (`<name>(<p,p>)`).
struct Entry {
    key: String,
    indicator: Box<dyn EvalIndicator>,
}

/// The set of indicators a feature spec needs, folded one candle at a time. Each
/// `update` records the primary value under the indicator's registry key and
/// every named sub-output under `<key>.<field>`.
pub struct IndicatorSet {
    items: Vec<Entry>,
    cur: BTreeMap<String, f64>,
}

impl IndicatorSet {
    /// An empty set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cur: BTreeMap::new(),
        }
    }

    /// Resolve every indicator/microstructure feature of a spec, or fail on the
    /// first unknown name. Price features need no indicator.
    ///
    /// # Errors
    /// Returns [`Error::UnknownIndicator`] if the registry does not know a
    /// referenced indicator or rejects its parameters.
    pub fn from_features(features: &[Feature]) -> Result<Self> {
        let mut set = Self::new();
        for feature in features {
            set.required(feature)?;
        }
        Ok(set)
    }

    /// Register the indicator a feature needs (price fields need none).
    /// Idempotent per registry key.
    ///
    /// # Errors
    /// Returns [`Error::UnknownIndicator`] if the registry rejects the name or
    /// parameters.
    pub fn required(&mut self, feature: &Feature) -> Result<()> {
        let (name, params) = match feature {
            Feature::Indicator { name, params, .. } => (name.as_str(), params.as_slice()),
            Feature::Microstructure { metric, params } => (metric.as_str(), params.as_slice()),
            Feature::Price { .. } => return Ok(()),
        };
        let key = registry_key(name, params);
        if self.items.iter().all(|e| e.key != key) {
            let indicator =
                build(name, params).map_err(|e| Error::UnknownIndicator(format!("{name}: {e}")))?;
            self.items.push(Entry { key, indicator });
        }
        Ok(())
    }

    /// Fold one candle: every indicator ticks and records its primary value and
    /// named fields into the current-bar map (cleared first).
    pub fn update(&mut self, candle: &Candle) {
        self.cur.clear();
        let bar = BarInput {
            candle,
            reference: None,
            deriv: None,
            orderbook: None,
            trades: &[],
            cross_section: None,
        };
        for entry in &mut self.items {
            if let Some(value) = entry.indicator.update(&bar) {
                self.cur.insert(entry.key.clone(), value);
                for (field, field_value) in entry.indicator.fields() {
                    self.cur
                        .insert(format!("{}.{field}", entry.key), field_value);
                }
            }
        }
    }

    /// The current value for a registry key (or `<key>.<field>`), if computed
    /// this bar.
    #[must_use]
    pub fn cur(&self, key: &str) -> Option<f64> {
        self.cur.get(key).copied()
    }
}

impl Default for IndicatorSet {
    fn default() -> Self {
        Self::new()
    }
}

/// The registry key for a feature's indicator lookup: `<name>(<p,p,...>)`,
/// without the `ms.` prefix or `.field` suffix the column key carries.
#[must_use]
pub fn registry_key(name: &str, params: &[f64]) -> String {
    format!("{name}({})", fmt_params(params))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(close: f64) -> Candle {
        Candle {
            time: 0,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
        }
    }

    #[test]
    fn resolves_and_folds_an_sma() {
        let mut set = IndicatorSet::new();
        set.required(&Feature::Indicator {
            name: "Sma".into(),
            params: vec![3.0],
            field: None,
        })
        .unwrap();
        for c in [1.0, 2.0, 3.0, 4.0, 5.0] {
            set.update(&candle(c));
        }
        assert_eq!(set.cur("Sma(3)"), Some(4.0));
    }

    #[test]
    fn price_feature_registers_nothing() {
        let set = IndicatorSet::from_features(&[Feature::Price {
            field: crate::feature::PriceField::Close,
        }])
        .unwrap();
        assert!(set.cur("anything").is_none());
    }

    #[test]
    fn unknown_indicator_errors() {
        let mut set = IndicatorSet::new();
        assert!(matches!(
            set.required(&Feature::Indicator {
                name: "NotAnIndicator".into(),
                params: vec![],
                field: None,
            }),
            Err(Error::UnknownIndicator(_))
        ));
    }
}
