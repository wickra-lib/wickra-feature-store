//! Per-symbol folding engine: ticks the indicator set one candle at a time and
//! exposes the current feature row in spec column order.
//!
//! A `SymbolState` keeps no bar history of its own — only the resolved
//! [`IndicatorSet`] and the current feature row. The candle history needed for
//! forward-looking labels lives in the [`crate::universe::Universe`]; the batch
//! [`crate::build::build`] re-folds a fresh `SymbolState` per symbol.

use crate::feature::{Feature, PriceField};
use crate::indicator_set::{registry_key, IndicatorSet};
use crate::Result;
use wickra_backtest_core::Candle;

/// The folding state for one symbol: the resolved indicators and the current
/// feature row (spec order, `NaN` where a feature is not yet ready).
pub struct SymbolState {
    inds: IndicatorSet,
    features: Vec<Feature>,
    cur_row: Vec<f64>,
    bars: usize,
}

impl SymbolState {
    /// Resolve every feature of a spec into a folding state.
    ///
    /// # Errors
    /// Returns [`crate::Error::UnknownIndicator`] if a referenced indicator is
    /// not in the registry.
    pub fn new(features: &[Feature]) -> Result<Self> {
        Ok(Self {
            inds: IndicatorSet::from_features(features)?,
            features: features.to_vec(),
            cur_row: vec![f64::NAN; features.len()],
            bars: 0,
        })
    }

    /// Fold one candle in O(1) per indicator: tick the set and recompute the
    /// current feature row.
    pub fn fold(&mut self, candle: &Candle) {
        self.inds.update(candle);
        self.cur_row = self
            .features
            .iter()
            .map(|f| feature_value(f, &self.inds, candle))
            .collect();
        self.bars += 1;
    }

    /// The current feature row in spec column order (`NaN` where not ready).
    #[must_use]
    pub fn feature_row(&self) -> Vec<f64> {
        self.cur_row.clone()
    }

    /// Whether every feature cell of the current row is ready (no `NaN`) — the
    /// gate for [`crate::spec::WarmupPolicy::Skip`].
    #[must_use]
    pub fn all_ready(&self) -> bool {
        !self.cur_row.iter().any(|x| x.is_nan())
    }

    /// The number of candles folded so far.
    #[must_use]
    pub fn bars(&self) -> usize {
        self.bars
    }
}

/// The value of one feature for the current bar: the raw price field, or the
/// indicator/microstructure value looked up by its registry key (`NaN` when the
/// indicator is not ready this bar).
fn feature_value(feature: &Feature, inds: &IndicatorSet, candle: &Candle) -> f64 {
    match feature {
        Feature::Price { field } => match field {
            PriceField::Open => candle.open,
            PriceField::High => candle.high,
            PriceField::Low => candle.low,
            PriceField::Close => candle.close,
            PriceField::Volume => candle.volume,
        },
        Feature::Indicator {
            name,
            params,
            field,
        } => {
            let key = registry_key(name, params);
            let lookup = match field {
                Some(f) => format!("{key}.{f}"),
                None => key,
            };
            inds.cur(&lookup).unwrap_or(f64::NAN)
        }
        Feature::Microstructure { metric, params } => {
            inds.cur(&registry_key(metric, params)).unwrap_or(f64::NAN)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(close: f64) -> Candle {
        Candle {
            time: 0,
            open: close - 1.0,
            high: close + 1.0,
            low: close - 2.0,
            close,
            volume: 42.0,
        }
    }

    #[test]
    fn price_features_ready_immediately() {
        let features = vec![
            Feature::Price {
                field: PriceField::Close,
            },
            Feature::Price {
                field: PriceField::Volume,
            },
        ];
        let mut st = SymbolState::new(&features).unwrap();
        st.fold(&candle(100.0));
        assert_eq!(st.feature_row(), vec![100.0, 42.0]);
        assert!(st.all_ready());
        assert_eq!(st.bars(), 1);
    }

    #[test]
    fn indicator_is_nan_during_warmup() {
        let features = vec![Feature::Indicator {
            name: "Sma".into(),
            params: vec![3.0],
            field: None,
        }];
        let mut st = SymbolState::new(&features).unwrap();
        st.fold(&candle(1.0));
        assert!(!st.all_ready());
        assert!(st.feature_row()[0].is_nan());
        st.fold(&candle(2.0));
        st.fold(&candle(3.0));
        assert!(st.all_ready());
        assert!((st.feature_row()[0] - 2.0).abs() < 1e-12);
    }
}
