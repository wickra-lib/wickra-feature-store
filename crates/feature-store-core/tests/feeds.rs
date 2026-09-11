//! The side feeds, end to end.
//!
//! Each family is exercised twice: once without its feed, where the spec must be
//! refused by name, and once with it, where the column must carry finite values.
//! Before the feeds existed, a build naming one of these indicators ran to
//! completion and emitted a column that was `NaN` for its whole length — while a
//! misspelt name, the far less likely mistake, failed loudly. That asymmetry is
//! what these tests pin shut.

use std::collections::BTreeMap;

use wickra_backtest_core::{CrossSection, CrossSectionMember, DerivativesTick, Level, OrderBook};
use wickra_backtest_core::{TradePrint, TradeSide};
use wickra_feature_store_core::{
    build_series, Candle, Error, Feature, FeatureSpec, PriceField, SymbolSeries, WarmupPolicy,
};

const SYMBOL: &str = "AAA";
const BARS: usize = 120;

/// The timestamp of bar `i`, converted rather than cast: the workspace denies
/// `usize as i64`.
fn bar_time(i: usize) -> i64 {
    1_700_000_000 + i64::try_from(i).expect("bar index fits an i64") * 3600
}

/// A varying price path. A geometric path has constant log-returns, which makes
/// the correlation family degenerate; this one does not.
fn candles() -> Vec<Candle> {
    (0..BARS)
        .map(|i| {
            let t = i as f64;
            let ts = bar_time(i);
            let close = 100.0 + (t * 0.35).sin() * 8.0 + t * 0.05;
            Candle {
                time: ts,
                open: close - 0.4,
                high: close + 1.2,
                low: close - 1.2,
                close,
                volume: 1_000.0 + t * 3.0,
            }
        })
        .collect()
}

/// A reference series that moves differently from the subject, so a pairwise
/// indicator has a non-degenerate relationship to measure.
fn reference() -> Vec<Candle> {
    (0..BARS)
        .map(|i| {
            let t = i as f64;
            let ts = bar_time(i);
            let close = 50.0 + (t * 0.22).cos() * 6.0 + t * 0.03;
            Candle {
                time: ts,
                open: close,
                high: close + 0.5,
                low: close - 0.5,
                close,
                volume: 500.0,
            }
        })
        .collect()
}

fn derivs() -> Vec<DerivativesTick> {
    (0..BARS)
        .map(|i| {
            let t = i as f64;
            let ts = bar_time(i);
            let mark = 100.0 + (t * 0.35).sin() * 8.0;
            DerivativesTick {
                funding_rate: 0.0001 + (t * 0.1).sin() * 0.000_05,
                mark_price: mark,
                index_price: mark - 0.02,
                futures_price: mark + 0.05,
                open_interest: 1_000_000.0 + t * 250.0,
                long_size: 600_000.0 + t * 120.0,
                short_size: 400_000.0 + t * 130.0,
                taker_buy_volume: 300.0 + t,
                taker_sell_volume: 280.0 + t,
                long_liquidation: t.mul_add(0.5, 10.0),
                short_liquidation: t.mul_add(0.4, 8.0),
                timestamp: ts,
            }
        })
        .collect()
}

fn books() -> Vec<OrderBook> {
    (0..BARS)
        .map(|i| {
            let mid = 100.0 + (i as f64 * 0.35).sin() * 8.0;
            OrderBook {
                bids: vec![
                    Level {
                        price: mid - 0.05,
                        size: 12.0,
                    },
                    Level {
                        price: mid - 0.10,
                        size: 20.0,
                    },
                ],
                asks: vec![
                    Level {
                        price: mid + 0.05,
                        size: 9.0,
                    },
                    Level {
                        price: mid + 0.10,
                        size: 18.0,
                    },
                ],
            }
        })
        .collect()
}

fn trades() -> Vec<Vec<TradePrint>> {
    (0..BARS)
        .map(|i| {
            let ts = bar_time(i);
            let mid = 100.0 + (i as f64 * 0.35).sin() * 8.0;
            vec![
                TradePrint {
                    price: mid + 0.05,
                    size: 3.0,
                    side: TradeSide::Buy,
                    timestamp: ts,
                },
                TradePrint {
                    price: mid - 0.05,
                    size: 2.0,
                    side: TradeSide::Sell,
                    timestamp: ts,
                },
            ]
        })
        .collect()
}

fn sections() -> Vec<CrossSection> {
    (0..BARS)
        .map(|i| {
            let t = i as f64;
            let ts = bar_time(i);
            CrossSection {
                members: (0..8)
                    .map(|k| {
                        let change = (t * 0.3 + f64::from(k)).sin();
                        CrossSectionMember {
                            change,
                            volume: 1_000.0 + f64::from(k) * 10.0,
                            new_high: change > 0.8,
                            new_low: change < -0.8,
                        }
                    })
                    .collect(),
                timestamp: ts,
            }
        })
        .collect()
}

/// A candle-only series — every feed absent.
fn bare() -> BTreeMap<String, SymbolSeries> {
    BTreeMap::from([(
        SYMBOL.to_string(),
        SymbolSeries {
            candles: candles(),
            ..SymbolSeries::default()
        },
    )])
}

/// A series with the feeds a closure switches on.
fn with(f: impl FnOnce(&mut SymbolSeries)) -> BTreeMap<String, SymbolSeries> {
    let mut series = SymbolSeries {
        candles: candles(),
        ..SymbolSeries::default()
    };
    f(&mut series);
    BTreeMap::from([(SYMBOL.to_string(), series)])
}

/// A spec with one indicator column plus the close, emitting every bar so a
/// dead column is visible rather than skipped by the warmup policy.
fn spec_for(name: &str, params: Vec<f64>) -> FeatureSpec {
    FeatureSpec {
        universe: vec![SYMBOL.to_string()],
        timeframe: Some("1h".to_string()),
        features: vec![
            Feature::Indicator {
                name: name.to_string(),
                params,
                field: None,
            },
            Feature::Price {
                field: PriceField::Close,
            },
        ],
        labels: Vec::new(),
        window: None,
        output: wickra_feature_store_core::OutputFormat::Json,
        scaling: None,
        warmup: WarmupPolicy::Nan,
    }
}

/// Build with the feed and assert the indicator column carries a finite value.
fn assert_column_is_alive(name: &str, params: Vec<f64>, data: &BTreeMap<String, SymbolSeries>) {
    let spec = spec_for(name, params);
    let matrix = build_series(data, &spec).unwrap_or_else(|e| panic!("{name} with its feed: {e}"));
    let json = matrix.to_json();
    let value: serde_json::Value = serde_json::from_str(&json).expect("matrix json");
    let rows = value["data"].as_array().expect("data rows");
    let finite = rows
        .iter()
        .filter(|r| r[0].as_f64().is_some_and(f64::is_finite))
        .count();
    assert!(
        finite > 0,
        "{name} produced {finite} finite values in {} rows even with its feed supplied",
        rows.len()
    );
}

/// The refusal check, written so the error is inspected rather than panicked on.
fn refusal(name: &str, params: Vec<f64>) -> Error {
    let spec = spec_for(name, params);
    build_series(&bare(), &spec).expect_err("must be refused without its feed")
}

fn assert_names_feed(name: &str, params: Vec<f64>, feed: &str) {
    match refusal(name, params) {
        Error::MissingFeed {
            indicator,
            feed: got,
        } => {
            assert_eq!(indicator, name);
            assert_eq!(got, feed);
        }
        other => panic!("{name}: expected MissingFeed, got {other}"),
    }
}

#[test]
fn pairwise_indicators_need_a_reference_series() {
    assert_names_feed("RollingCorrelation", vec![20.0], "reference");
    assert_column_is_alive(
        "RollingCorrelation",
        vec![20.0],
        &with(|s| s.reference = Some(reference())),
    );
}

#[test]
fn derivatives_indicators_need_a_derivatives_tick() {
    assert_names_feed("FundingRate", vec![], "derivs");
    assert_column_is_alive("FundingRate", vec![], &with(|s| s.derivs = Some(derivs())));
}

#[test]
fn order_book_indicators_need_a_book() {
    assert_names_feed("Microprice", vec![], "books");
    assert_column_is_alive("Microprice", vec![], &with(|s| s.books = Some(books())));
}

#[test]
fn trade_flow_indicators_need_trades() {
    assert_names_feed("CumulativeVolumeDelta", vec![], "trades");
    assert_column_is_alive(
        "CumulativeVolumeDelta",
        vec![],
        &with(|s| s.trades = Some(trades())),
    );
}

#[test]
fn trade_quote_indicators_need_trades_and_a_book() {
    assert_names_feed("EffectiveSpread", vec![], "trades and books");
    // Trades alone are not enough: the quote half is still missing.
    let spec = spec_for("EffectiveSpread", vec![]);
    let only_trades = with(|s| s.trades = Some(trades()));
    build_series(&only_trades, &spec).expect_err("EffectiveSpread without a book must be refused");
    assert_column_is_alive(
        "EffectiveSpread",
        vec![],
        &with(|s| {
            s.trades = Some(trades());
            s.books = Some(books());
        }),
    );
}

#[test]
fn breadth_indicators_read_a_cross_section() {
    assert_names_feed("AdvanceDecline", vec![], "sections");
    assert_column_is_alive(
        "AdvanceDecline",
        vec![],
        &with(|s| s.sections = Some(sections())),
    );
}

#[test]
fn a_candle_only_indicator_still_needs_nothing() {
    let spec = spec_for("Rsi", vec![14.0]);
    let matrix = build_series(&bare(), &spec).expect("Rsi builds on candles alone");
    assert!(matrix.rows > 0);
}

#[test]
fn a_microstructure_metric_is_checked_like_any_other_column() {
    let spec = FeatureSpec {
        features: vec![Feature::Microstructure {
            metric: "Microprice".to_string(),
            params: vec![],
        }],
        ..spec_for("Rsi", vec![14.0])
    };
    match build_series(&bare(), &spec).expect_err("a fed metric with no feed must be refused") {
        Error::MissingFeed { indicator, feed } => {
            assert_eq!(indicator, "Microprice");
            assert_eq!(feed, "books");
        }
        other => panic!("expected MissingFeed, got {other}"),
    }
}

#[test]
fn a_feed_shorter_than_the_candles_is_refused_by_length() {
    let spec = spec_for("RollingCorrelation", vec![20.0]);
    let short = with(|s| s.reference = Some(reference()[..10].to_vec()));
    let err = build_series(&short, &spec).expect_err("a short feed must be refused");
    assert!(
        err.to_string().contains("reference feed length 10"),
        "the error names the feed and both lengths: {err}"
    );
}

/// Every name treated as pairwise really does need a reference: it is refused on
/// candles alone and yields values once the reference arrives.
///
/// This is the half of the drift check that can be written today. The other
/// half — proving no *other* registry name needs a reference — would have to
/// enumerate the catalogue, and `wickra-backtest-core` exposes no such listing.
#[test]
fn every_pairwise_name_behaves_like_one() {
    const PAIRWISE: [(&str, &[f64]); 24] = [
        // Parameters follow each indicator's own constructor: a period, or a
        // period plus its second argument (an ADF lag, a z-score window, a
        // std-dev multiplier, a risk-free rate, a Kalman delta).
        ("Alpha", &[20.0, 0.0]),
        ("Beta", &[20.0]),
        ("BetaNeutralSpread", &[20.0]),
        ("Cointegration", &[30.0, 1.0]),
        ("DistanceSsd", &[20.0]),
        ("GrangerCausality", &[30.0, 2.0]),
        ("HasbrouckInformationShare", &[20.0]),
        ("InformationRatio", &[20.0]),
        ("KalmanHedgeRatio", &[0.0001, 0.001]),
        ("KendallTau", &[20.0]),
        ("LeadLagCrossCorrelation", &[20.0, 3.0]),
        ("OuHalfLife", &[30.0]),
        ("PairSpreadZScore", &[20.0, 20.0]),
        ("PairwiseBeta", &[20.0]),
        ("PearsonCorrelation", &[20.0]),
        ("RelativeStrengthAB", &[20.0, 14.0]),
        ("RollingCorrelation", &[20.0]),
        ("RollingCovariance", &[20.0]),
        ("SpearmanCorrelation", &[20.0]),
        ("SpreadAr1Coefficient", &[20.0]),
        ("SpreadBollingerBands", &[20.0, 2.0]),
        ("SpreadHurst", &[30.0]),
        ("TreynorRatio", &[20.0, 0.0]),
        ("VarianceRatio", &[20.0, 4.0]),
    ];

    for (name, params) in PAIRWISE {
        assert_names_feed(name, params.to_vec(), "reference");
        assert_column_is_alive(
            name,
            params.to_vec(),
            &with(|s| s.reference = Some(reference())),
        );
    }
}
