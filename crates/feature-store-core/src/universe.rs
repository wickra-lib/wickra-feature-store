//! The pushed candle history per symbol — the streaming state a
//! [`crate::feature_store::FeatureStore`] accumulates.
//!
//! Symbols are keyed in a `BTreeMap` so emission order is deterministic (sorted
//! by symbol key) regardless of push order. Forward-looking labels need the full
//! candle history, so the candles are retained; [`crate::build::build_series`]
//! re-folds them per symbol.
//!
//! A push may carry the bar's side feeds. Each feed is accumulated into its own
//! parallel array, so a feed that arrives on some bars and not others produces a
//! feed shorter than the candle history — which
//! [`crate::feeds::CoreSeries::build`] rejects by length, naming the symbol and
//! the feed. That is deliberate: back-filling the missing bars would put back
//! exactly the silent hole the feeds exist to close.

use crate::feeds::SymbolSeries;
use std::collections::BTreeMap;
use wickra_backtest_core::{Candle, CrossSection, DerivativesTick, OrderBook, TradePrint};

/// One bar's side feeds in their JSON input form, as a streaming push carries
/// them. Mirrors `wickra_backtest_core::StepFeeds` field for field.
#[derive(Default, Clone, Debug)]
pub struct PushFeeds {
    /// Reference-series close for the pairwise family.
    pub reference: Option<f64>,
    /// The bar's derivatives tick.
    pub deriv: Option<DerivativesTick>,
    /// The bar's order-book snapshot.
    pub orderbook: Option<OrderBook>,
    /// The trades that printed within the bar.
    pub trades: Option<Vec<TradePrint>>,
    /// The market cross-section at the bar.
    pub cross_section: Option<CrossSection>,
}

impl PushFeeds {
    /// Whether this push carries no feed at all — the candle-only case.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reference.is_none()
            && self.deriv.is_none()
            && self.orderbook.is_none()
            && self.trades.is_none()
            && self.cross_section.is_none()
    }
}

/// The candle history of every symbol pushed so far, keyed for deterministic
/// (sorted) emission order, with each symbol's accumulated side feeds.
#[derive(Default, Clone)]
pub struct Universe {
    series: BTreeMap<String, SymbolSeries>,
}

impl Universe {
    /// An empty universe.
    #[must_use]
    pub fn new() -> Self {
        Self {
            series: BTreeMap::new(),
        }
    }

    /// Append one candle to a symbol's history (creating the symbol on first
    /// push), carrying no side feeds.
    pub fn push(&mut self, symbol: &str, candle: Candle) {
        self.push_with(symbol, candle, &PushFeeds::default());
    }

    /// Append one candle and its side feeds to a symbol's history.
    ///
    /// A feed present on this push is appended to that feed's array; a feed
    /// absent leaves its array untouched, so mixing carried and uncarried bars
    /// leaves a short feed rather than a fabricated one.
    pub fn push_with(&mut self, symbol: &str, candle: Candle, feeds: &PushFeeds) {
        let series = self.series.entry(symbol.to_string()).or_default();
        series.candles.push(candle);
        if let Some(reference) = feeds.reference {
            // The series form carries reference candles; only the close is read,
            // so a bare close is widened onto the bar's own timestamp.
            series.reference.get_or_insert_with(Vec::new).push(Candle {
                time: candle.time,
                open: reference,
                high: reference,
                low: reference,
                close: reference,
                volume: 0.0,
            });
        }
        if let Some(deriv) = feeds.deriv {
            series.derivs.get_or_insert_with(Vec::new).push(deriv);
        }
        if let Some(book) = feeds.orderbook.clone() {
            series.books.get_or_insert_with(Vec::new).push(book);
        }
        if let Some(trades) = feeds.trades.clone() {
            series.trades.get_or_insert_with(Vec::new).push(trades);
        }
        if let Some(section) = feeds.cross_section.clone() {
            series.sections.get_or_insert_with(Vec::new).push(section);
        }
    }

    /// Append many candles to a symbol's history, carrying no side feeds.
    pub fn push_batch(&mut self, symbol: &str, candles: &[Candle]) {
        self.series
            .entry(symbol.to_string())
            .or_default()
            .candles
            .extend_from_slice(candles);
    }

    /// Drop all candles and feeds (used by the `reset` command).
    pub fn clear(&mut self) {
        self.series.clear();
    }

    /// Whether nothing has been pushed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    /// The backing map, symbol-sorted — the input to
    /// [`crate::build::build_series`].
    #[must_use]
    pub fn data(&self) -> &BTreeMap<String, SymbolSeries> {
        &self.series
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(t: i64) -> Candle {
        Candle {
            time: t,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 0.0,
        }
    }

    #[test]
    fn push_accumulates_in_sorted_symbol_order() {
        let mut u = Universe::new();
        assert!(u.is_empty());
        u.push("BBB", candle(2));
        u.push("AAA", candle(1));
        u.push_batch("AAA", &[candle(3), candle(4)]);
        let keys: Vec<&String> = u.data().keys().collect();
        assert_eq!(keys, vec!["AAA", "BBB"]);
        assert_eq!(u.data()["AAA"].candles.len(), 3);
        u.clear();
        assert!(u.is_empty());
    }

    #[test]
    fn a_carried_reference_accumulates_alongside_the_candles() {
        let mut u = Universe::new();
        let feeds = PushFeeds {
            reference: Some(42.0),
            ..PushFeeds::default()
        };
        u.push_with("AAA", candle(1), &feeds);
        u.push_with("AAA", candle(2), &feeds);
        let series = &u.data()["AAA"];
        assert_eq!(series.candles.len(), 2);
        let reference = series.reference.as_ref().expect("reference accumulated");
        assert_eq!(reference.len(), 2);
        assert!((reference[0].close - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_feed_carried_on_only_some_bars_stays_short() {
        let mut u = Universe::new();
        u.push_with(
            "AAA",
            candle(1),
            &PushFeeds {
                reference: Some(1.0),
                ..PushFeeds::default()
            },
        );
        u.push("AAA", candle(2));
        let series = &u.data()["AAA"];
        assert_eq!(series.candles.len(), 2);
        assert_eq!(
            series.reference.as_ref().map(Vec::len),
            Some(1),
            "the short feed is left short, for the length check to refuse"
        );
    }

    #[test]
    fn empty_push_feeds_report_empty() {
        assert!(PushFeeds::default().is_empty());
        assert!(!PushFeeds {
            reference: Some(1.0),
            ..PushFeeds::default()
        }
        .is_empty());
    }
}
