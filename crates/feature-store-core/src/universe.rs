//! The pushed candle history per symbol — the streaming state a
//! [`crate::feature_store::FeatureStore`] accumulates.
//!
//! Symbols are keyed in a `BTreeMap` so emission order is deterministic (sorted
//! by symbol key) regardless of push order. Forward-looking labels need the full
//! candle history, so the candles are retained; [`crate::build::build`] re-folds
//! them per symbol.

use std::collections::BTreeMap;
use wickra_backtest_core::Candle;

/// The candle history of every symbol pushed so far, keyed for deterministic
/// (sorted) emission order.
#[derive(Default, Clone)]
pub struct Universe {
    candles: BTreeMap<String, Vec<Candle>>,
}

impl Universe {
    /// An empty universe.
    #[must_use]
    pub fn new() -> Self {
        Self {
            candles: BTreeMap::new(),
        }
    }

    /// Append one candle to a symbol's history (creating the symbol on first
    /// push).
    pub fn push(&mut self, symbol: &str, candle: Candle) {
        self.candles
            .entry(symbol.to_string())
            .or_default()
            .push(candle);
    }

    /// Append many candles to a symbol's history.
    pub fn push_batch(&mut self, symbol: &str, candles: &[Candle]) {
        self.candles
            .entry(symbol.to_string())
            .or_default()
            .extend_from_slice(candles);
    }

    /// Drop all candles (used by the `reset` command).
    pub fn clear(&mut self) {
        self.candles.clear();
    }

    /// Whether no candles have been pushed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// The backing map, symbol-sorted — the input to [`crate::build::build`].
    #[must_use]
    pub fn data(&self) -> &BTreeMap<String, Vec<Candle>> {
        &self.candles
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
        assert_eq!(u.data()["AAA"].len(), 3);
        u.clear();
        assert!(u.is_empty());
    }
}
