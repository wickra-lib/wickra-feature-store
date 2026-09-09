//! Resolves the indicators a spec references and folds candles through them.
//!
//! Indicators are resolved by name and parameters from the `wickra-core`
//! registry, reused through the `wickra-backtest-core` factory — the only
//! name -> indicator resolver in the ecosystem. Each resolved indicator is an
//! object-safe `EvalIndicator`, driven with a [`BarInput`] carrying the candle
//! and whatever side feeds the caller supplied — a reference close, a
//! derivatives tick, an order book, the bar's trades, the market cross-section.
//! Microstructure metrics resolve through the very same registry, only from the
//! microstructure namespace, so they share this code path and read the same
//! feeds.
//!
//! [`feed_kind`] answers which feed a name consumes, which is what lets
//! [`crate::spec::FeatureSpec::check_feeds`] refuse a spec whose columns could
//! only ever be `NaN`.

use crate::error::{Error, Result};
use crate::feature::{fmt_params, Feature};
use crate::feeds::{BarFeeds, FeedKind};
use std::collections::BTreeMap;
use wickra_backtest_core::registry::{build, feed_of, BarInput};
use wickra_backtest_core::spec::Feed;
use wickra_backtest_core::{Candle, EvalIndicator};

/// The pairwise indicators, which read the reference series' close alongside the
/// bar close.
///
/// This list exists because `registry::feed_of` cannot express the family:
/// upstream classifies every pairwise indicator as `Feed::Kline`, since the
/// candle is indeed one of its two inputs. The `pairwise_list_matches_behaviour`
/// test probes every name here against the live registry and fails if the list
/// ever drifts from the set that actually needs a reference, so a registry that
/// grows a new pairwise indicator cannot slip past silently.
const PAIRWISE: [&str; 24] = [
    "Alpha",
    "Beta",
    "BetaNeutralSpread",
    "Cointegration",
    "DistanceSsd",
    "GrangerCausality",
    "HasbrouckInformationShare",
    "InformationRatio",
    "KalmanHedgeRatio",
    "KendallTau",
    "LeadLagCrossCorrelation",
    "OuHalfLife",
    "PairSpreadZScore",
    "PairwiseBeta",
    "PearsonCorrelation",
    "RelativeStrengthAB",
    "RollingCorrelation",
    "RollingCovariance",
    "SpearmanCorrelation",
    "SpreadAr1Coefficient",
    "SpreadBollingerBands",
    "SpreadHurst",
    "TreynorRatio",
    "VarianceRatio",
];

/// Which feed an indicator consumes, or `None` if the registry does not know it.
///
/// Wraps `registry::feed_of` and refines its `Kline` answer with the pairwise
/// list, so a caller can tell "candle is enough" from "needs a reference".
#[must_use]
pub fn feed_kind(name: &str) -> Option<FeedKind> {
    let feed = feed_of(name)?;
    Some(match feed {
        Feed::Kline if PAIRWISE.contains(&name) => FeedKind::Pair,
        Feed::Kline => FeedKind::Candle,
        Feed::Trade => FeedKind::Trades,
        Feed::Orderbook => FeedKind::OrderBook,
        Feed::TradeQuote => FeedKind::TradeQuote,
        Feed::Derivatives => FeedKind::Derivatives,
        Feed::CrossSection => FeedKind::CrossSection,
    })
}

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

    /// Fold one candle and its side feeds: every indicator ticks and records its
    /// primary value and named fields into the current-bar map (cleared first).
    pub fn update(&mut self, candle: &Candle, feeds: BarFeeds<'_>) {
        self.cur.clear();
        let bar = BarInput {
            candle,
            reference: feeds.reference,
            deriv: feeds.deriv,
            orderbook: feeds.orderbook,
            trades: feeds.trades,
            cross_section: feeds.cross_section,
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
            set.update(&candle(c), BarFeeds::default());
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
