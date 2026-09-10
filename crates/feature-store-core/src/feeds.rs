//! The side feeds an indicator may consume beyond the candle.
//!
//! Most of the registry is driven by the candle alone, but a substantial part of
//! it reads something else: a reference series (the pairwise family), a
//! derivatives tick, an order-book snapshot, the trades that printed within the
//! bar, or the market cross-section. An indicator whose feed is absent resolves,
//! ticks and returns nothing — every bar, without complaint — so a feature
//! column naming one would be `NaN` for its whole length while an unknown name,
//! the one mistake a caller is far less likely to make, fails loudly.
//!
//! Supplying the feeds is what makes those indicators reachable; the feed check
//! in [`crate::spec::FeatureSpec::check_feeds`] rejects the specs whose feeds a
//! build cannot supply, naming the column and the feed, so the silent case is
//! gone in both directions.
//!
//! The shapes mirror `wickra-backtest`: [`SymbolSeries`] is the batch form —
//! parallel arrays, one entry per candle, exactly like its `RunRequest` — and
//! `wickra_backtest_core::StepFeeds` is the per-bar streaming form, reused
//! verbatim so the JSON field names match across the ecosystem. A caller holding
//! only candles keeps passing a bare array.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use wickra_backtest_core::{
    Candle, CrossSection, DerivativesTick, OrderBook, StepFeeds, TradePrint,
};
use wickra_core::{
    CrossSection as CoreCrossSection, DerivativesTick as CoreDerivativesTick,
    OrderBook as CoreOrderBook, Trade as CoreTrade,
};

/// Which side feed an indicator family consumes.
///
/// This mirrors `wickra_backtest_core::spec::Feed` with one addition: the
/// pairwise family. Upstream classifies pairwise indicators as `Kline` because
/// they are fed the bar close alongside the reference close, which leaves no way
/// to express "needs a reference series" — [`FeedKind::Pair`] is that missing
/// case, and [`crate::indicator_set::feed_kind`] maps a name onto it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedKind {
    /// The candle alone.
    Candle,
    /// The candle plus the reference series' close (pairwise indicators).
    Pair,
    /// A derivatives tick — funding, open interest, mark and index.
    Derivatives,
    /// An order-book snapshot.
    OrderBook,
    /// The trades that printed within the bar.
    Trades,
    /// Trades quoted against the book mid (needs both feeds).
    TradeQuote,
    /// The market cross-section, for the breadth family.
    CrossSection,
}

impl FeedKind {
    /// The name used in error messages and documentation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FeedKind::Candle => "candle",
            FeedKind::Pair => "reference",
            FeedKind::Derivatives => "derivs",
            FeedKind::OrderBook => "books",
            FeedKind::Trades => "trades",
            FeedKind::TradeQuote => "trades and books",
            FeedKind::CrossSection => "sections",
        }
    }
}

/// One symbol's candle history plus its optional side feeds.
///
/// Every supplied feed is as long as `candles`; a shorter or longer one is an
/// error rather than a feed that quietly runs out part-way through the fold.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SymbolSeries {
    /// The candle history, oldest first.
    pub candles: Vec<Candle>,
    /// The reference series for pairwise indicators; its close is what they read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<Vec<Candle>>,
    /// One derivatives tick per candle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivs: Option<Vec<DerivativesTick>>,
    /// One order-book snapshot per candle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub books: Option<Vec<OrderBook>>,
    /// The trades that printed within each candle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<Vec<TradePrint>>>,
    /// One market cross-section per candle, for the breadth family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<CrossSection>>,
}

/// A candle in its JSON input form. The wire contract calls the timestamp `ts`;
/// the core `Candle` calls the field `time`, so the two spellings meet here
/// rather than in every caller.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct CandleInput {
    /// Bar timestamp.
    pub ts: i64,
    /// Opening price.
    pub open: f64,
    /// Highest price.
    pub high: f64,
    /// Lowest price.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Traded volume.
    pub volume: f64,
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

/// The full per-symbol series in its JSON input form: parallel arrays, one entry
/// per candle, mirroring `wickra-backtest`'s `RunRequest`.
#[derive(Deserialize, Clone, Debug)]
pub struct SeriesDoc {
    /// The candle history, oldest first.
    pub candles: Vec<CandleInput>,
    /// The reference series for the pairwise family.
    #[serde(default)]
    pub reference: Option<Vec<CandleInput>>,
    /// One derivatives tick per candle.
    #[serde(default)]
    pub derivs: Option<Vec<DerivativesTick>>,
    /// One order-book snapshot per candle.
    #[serde(default)]
    pub books: Option<Vec<OrderBook>>,
    /// The trades that printed within each candle.
    #[serde(default)]
    pub trades: Option<Vec<Vec<TradePrint>>>,
    /// One market cross-section per candle.
    #[serde(default)]
    pub sections: Option<Vec<CrossSection>>,
}

/// One symbol's JSON input: a bare candle array, or the full series with feeds.
///
/// The bare form is the shorthand every candle-only caller and golden dataset
/// uses, so adding feeds costs nothing to a build that does not need them.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum SymbolInputDoc {
    /// Candles only.
    Candles(Vec<CandleInput>),
    /// Candles plus side feeds.
    Series(Box<SeriesDoc>),
}

fn candles_of(input: Vec<CandleInput>) -> Vec<Candle> {
    input.into_iter().map(Into::into).collect()
}

impl From<SymbolInputDoc> for SymbolSeries {
    fn from(doc: SymbolInputDoc) -> Self {
        match doc {
            SymbolInputDoc::Candles(candles) => SymbolSeries {
                candles: candles_of(candles),
                ..SymbolSeries::default()
            },
            SymbolInputDoc::Series(series) => SymbolSeries {
                candles: candles_of(series.candles),
                reference: series.reference.map(candles_of),
                derivs: series.derivs,
                books: series.books,
                trades: series.trades,
                sections: series.sections,
            },
        }
    }
}

/// A symbol's build input: a bare candle array, or the full series with feeds.
///
/// The bare form is the shorthand every existing caller and golden spec uses;
/// keeping it means adding feeds costs nothing to a build that does not need
/// them.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum SymbolInput {
    /// Candles only.
    Candles(Vec<Candle>),
    /// Candles plus side feeds.
    Series(Box<SymbolSeries>),
}

impl SymbolInput {
    /// The candle history behind this input, whichever form it took.
    #[must_use]
    pub fn candles(&self) -> &[Candle] {
        match self {
            SymbolInput::Candles(candles) => candles,
            SymbolInput::Series(series) => &series.candles,
        }
    }

    /// The series behind this input, with the bare form widened.
    #[must_use]
    pub fn to_series(&self) -> SymbolSeries {
        match self {
            SymbolInput::Candles(candles) => SymbolSeries {
                candles: candles.clone(),
                ..SymbolSeries::default()
            },
            SymbolInput::Series(series) => (**series).clone(),
        }
    }
}

impl From<Vec<Candle>> for SymbolInput {
    fn from(candles: Vec<Candle>) -> Self {
        SymbolInput::Candles(candles)
    }
}

impl From<SymbolSeries> for SymbolInput {
    fn from(series: SymbolSeries) -> Self {
        SymbolInput::Series(Box::new(series))
    }
}

/// A symbol's series with every feed converted to the `wickra-core` types the
/// indicators actually consume.
///
/// The conversion happens once, on ingest, so a malformed book or tick is an
/// error the caller sees. The backtester drops those with `.ok()`; here that
/// would put back exactly the silent hole this module exists to close.
#[derive(Debug)]
pub(crate) struct CoreSeries {
    pub(crate) candles: Vec<Candle>,
    reference: Option<Vec<f64>>,
    derivs: Option<Vec<CoreDerivativesTick>>,
    books: Option<Vec<CoreOrderBook>>,
    trades: Option<Vec<Vec<CoreTrade>>>,
    sections: Option<Vec<CoreCrossSection>>,
}

impl CoreSeries {
    /// Validate a series' feed lengths and convert every feed to its core type.
    ///
    /// # Errors
    /// [`Error::Data`] if a feed's length differs from the candle count, or if a
    /// feed entry is malformed.
    pub(crate) fn build(symbol: &str, series: SymbolSeries) -> Result<Self> {
        let n = series.candles.len();
        check_len(
            symbol,
            "reference",
            series.reference.as_ref().map(Vec::len),
            n,
        )?;
        check_len(symbol, "derivs", series.derivs.as_ref().map(Vec::len), n)?;
        check_len(symbol, "books", series.books.as_ref().map(Vec::len), n)?;
        check_len(symbol, "trades", series.trades.as_ref().map(Vec::len), n)?;
        check_len(
            symbol,
            "sections",
            series.sections.as_ref().map(Vec::len),
            n,
        )?;

        let reference = series
            .reference
            .map(|r| r.into_iter().map(|c| c.close).collect());
        let derivs = convert(symbol, series.derivs, DerivativesTick::to_core)?;
        let books = convert(symbol, series.books, |b| b.to_core())?;
        let sections = convert(symbol, series.sections, |s| s.to_core())?;
        let trades = series
            .trades
            .map(|bars| {
                bars.into_iter()
                    .map(|bar| convert_vec(symbol, bar, TradePrint::to_core))
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        Ok(Self {
            candles: series.candles,
            reference,
            derivs,
            books,
            trades,
            sections,
        })
    }

    /// Which feeds this series carries, for the spec's feed check.
    pub(crate) fn available(&self) -> Available {
        Available {
            reference: self.reference.is_some(),
            derivs: self.derivs.is_some(),
            books: self.books.is_some(),
            trades: self.trades.is_some(),
            sections: self.sections.is_some(),
        }
    }

    /// The feeds for bar `index`.
    pub(crate) fn bar(&self, index: usize) -> BarFeeds<'_> {
        BarFeeds {
            reference: self.reference.as_ref().map(|r| r[index]),
            deriv: self.derivs.as_ref().map(|d| d[index]),
            orderbook: self.books.as_ref().map(|b| &b[index]),
            trades: self
                .trades
                .as_ref()
                .map_or(&[][..], |t| t[index].as_slice()),
            cross_section: self.sections.as_ref().map(|s| &s[index]),
        }
    }
}

/// The feeds for one bar, in the types `BarInput` consumes.
#[derive(Debug, Default, Clone, Copy)]
pub struct BarFeeds<'a> {
    pub(crate) reference: Option<f64>,
    pub(crate) deriv: Option<CoreDerivativesTick>,
    pub(crate) orderbook: Option<&'a CoreOrderBook>,
    pub(crate) trades: &'a [CoreTrade],
    pub(crate) cross_section: Option<&'a CoreCrossSection>,
}

impl BarFeeds<'_> {
    /// Which feeds this bar carries.
    #[must_use]
    pub fn available(&self) -> Available {
        Available {
            reference: self.reference.is_some(),
            derivs: self.deriv.is_some(),
            books: self.orderbook.is_some(),
            trades: !self.trades.is_empty(),
            sections: self.cross_section.is_some(),
        }
    }
}

/// One bar's feeds owned rather than borrowed, so a streaming caller can hand
/// them in a command payload and have them outlive the deserialized document.
#[derive(Debug, Default)]
pub struct OwnedBarFeeds {
    reference: Option<f64>,
    deriv: Option<CoreDerivativesTick>,
    orderbook: Option<CoreOrderBook>,
    trades: Vec<CoreTrade>,
    cross_section: Option<CoreCrossSection>,
}

impl OwnedBarFeeds {
    /// Convert a streaming `StepFeeds` document, surfacing malformed feeds.
    ///
    /// # Errors
    /// [`Error::Data`] if a feed entry is malformed.
    pub fn build(symbol: &str, feeds: StepFeeds) -> Result<Self> {
        Ok(Self {
            reference: feeds.reference,
            deriv: feeds
                .deriv
                .map(|d| d.to_core().map_err(|e| feed_error(symbol, "derivs", &e)))
                .transpose()?,
            orderbook: feeds
                .orderbook
                .map(|b| b.to_core().map_err(|e| feed_error(symbol, "books", &e)))
                .transpose()?,
            trades: convert_vec(
                symbol,
                feeds.trades.unwrap_or_default(),
                TradePrint::to_core,
            )?,
            cross_section: feeds
                .cross_section
                .map(|s| s.to_core().map_err(|e| feed_error(symbol, "sections", &e)))
                .transpose()?,
        })
    }

    /// Borrow as the per-bar view.
    #[must_use]
    pub fn as_bar(&self) -> BarFeeds<'_> {
        BarFeeds {
            reference: self.reference,
            deriv: self.deriv,
            orderbook: self.orderbook.as_ref(),
            trades: &self.trades,
            cross_section: self.cross_section.as_ref(),
        }
    }
}

/// Which feed families are present, for the spec's feed check.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "one independent flag per feed family; a bitset would read worse at every use site"
)]
pub struct Available {
    /// A reference series (or per-bar reference close) is present.
    pub reference: bool,
    /// Derivatives ticks are present.
    pub derivs: bool,
    /// Order-book snapshots are present.
    pub books: bool,
    /// Trade prints are present.
    pub trades: bool,
    /// Market cross-sections are present.
    pub sections: bool,
}

impl Available {
    /// Every feed present — the availability a caller supplying the full bundle
    /// has, used when a spec is checked before any data is in hand.
    #[must_use]
    pub fn all() -> Self {
        Self {
            reference: true,
            derivs: true,
            books: true,
            trades: true,
            sections: true,
        }
    }

    /// Whether a feed family is present.
    #[must_use]
    pub fn has(self, kind: FeedKind) -> bool {
        match kind {
            FeedKind::Candle => true,
            FeedKind::Pair => self.reference,
            FeedKind::Derivatives => self.derivs,
            FeedKind::OrderBook => self.books,
            FeedKind::Trades => self.trades,
            FeedKind::TradeQuote => self.trades && self.books,
            FeedKind::CrossSection => self.sections,
        }
    }
}

/// A feed's length must equal the candle count, or it would quietly run out.
fn check_len(symbol: &str, name: &str, len: Option<usize>, n: usize) -> Result<()> {
    match len {
        Some(l) if l != n => Err(Error::Data(format!(
            "{symbol}: {name} feed length {l} does not match {n} candles"
        ))),
        _ => Ok(()),
    }
}

/// Convert an optional feed vector, surfacing the first malformed entry.
fn convert<T, U, E: core::fmt::Display>(
    symbol: &str,
    feed: Option<Vec<T>>,
    to_core: impl Fn(T) -> core::result::Result<U, E>,
) -> Result<Option<Vec<U>>> {
    feed.map(|items| convert_vec(symbol, items, &to_core))
        .transpose()
}

/// Convert a feed vector, surfacing the first malformed entry.
fn convert_vec<T, U, E: core::fmt::Display>(
    symbol: &str,
    items: Vec<T>,
    to_core: impl Fn(T) -> core::result::Result<U, E>,
) -> Result<Vec<U>> {
    items
        .into_iter()
        .map(|item| to_core(item).map_err(|e| feed_error(symbol, "feed", &e)))
        .collect()
}

/// A malformed feed entry, named by symbol and feed.
fn feed_error(symbol: &str, name: &str, e: &impl core::fmt::Display) -> Error {
    Error::Data(format!("{symbol}: malformed {name} entry: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(close: f64) -> Candle {
        Candle {
            time: 0,
            open: close,
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume: 1.0,
        }
    }

    #[test]
    fn a_bare_candle_array_widens_to_a_series_with_no_feeds() {
        let input = SymbolInput::from(vec![candle(1.0), candle(2.0)]);
        let series = input.to_series();
        assert_eq!(series.candles.len(), 2);
        assert!(series.reference.is_none());
        assert!(series.derivs.is_none());
        assert!(series.books.is_none());
        assert!(series.trades.is_none());
        assert!(series.sections.is_none());
    }

    #[test]
    fn a_short_feed_is_an_error_rather_than_a_fold_that_runs_out() {
        let series = SymbolSeries {
            candles: vec![candle(1.0), candle(2.0)],
            reference: Some(vec![candle(1.0)]),
            ..SymbolSeries::default()
        };
        let err = CoreSeries::build("AAA", series).expect_err("length mismatch must be refused");
        assert!(
            err.to_string().contains("reference feed length 1"),
            "error names the feed and both lengths: {err}"
        );
    }

    #[test]
    fn a_supplied_reference_reports_as_available_and_reaches_the_bar() {
        let series = SymbolSeries {
            candles: vec![candle(1.0), candle(2.0)],
            reference: Some(vec![candle(10.0), candle(20.0)]),
            ..SymbolSeries::default()
        };
        let core = CoreSeries::build("AAA", series).expect("build");
        assert!(core.available().has(FeedKind::Pair));
        assert!(!core.available().has(FeedKind::Derivatives));
        assert_eq!(core.bar(1).reference, Some(20.0));
    }

    #[test]
    fn trade_quote_needs_both_trades_and_a_book() {
        let both = Available {
            trades: true,
            books: true,
            ..Available::default()
        };
        let only_trades = Available {
            trades: true,
            ..Available::default()
        };
        assert!(both.has(FeedKind::TradeQuote));
        assert!(!only_trades.has(FeedKind::TradeQuote));
    }

    #[test]
    fn the_candle_feed_is_always_available() {
        assert!(Available::default().has(FeedKind::Candle));
    }

    #[test]
    fn every_feed_kind_renders_a_name() {
        for (kind, name) in [
            (FeedKind::Candle, "candle"),
            (FeedKind::Pair, "reference"),
            (FeedKind::Derivatives, "derivs"),
            (FeedKind::OrderBook, "books"),
            (FeedKind::Trades, "trades"),
            (FeedKind::TradeQuote, "trades and books"),
            (FeedKind::CrossSection, "sections"),
        ] {
            assert_eq!(kind.as_str(), name);
        }
    }

    #[test]
    fn all_reports_every_family_present() {
        let all = Available::all();
        for kind in [
            FeedKind::Pair,
            FeedKind::Derivatives,
            FeedKind::OrderBook,
            FeedKind::Trades,
            FeedKind::TradeQuote,
            FeedKind::CrossSection,
        ] {
            assert!(all.has(kind), "{} must be available", kind.as_str());
        }
    }
}
