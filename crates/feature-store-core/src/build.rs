//! The batch build entry: fold every symbol's candles into feature rows, join
//! forward-looking labels, apply the warmup policy, trailing window and optional
//! scaling, and materialize one [`FeatureMatrix`].
//!
//! Per-symbol folds are independent, so they run in parallel (the `parallel`
//! feature, rayon) and are merged back in symbol-sorted order — byte-for-byte
//! identical to the sequential WASM path, because the merge order and the serial
//! scaling pass never depend on thread scheduling.

use crate::error::Result;
use crate::indicator_set::IndicatorSet;
use crate::label::{forward_return, triple_barrier, Label};
use crate::matrix::{FeatureMatrix, RowId};
use crate::scaling::apply_scaling;
use crate::spec::{FeatureSpec, WarmupPolicy};
use crate::symbol_state::SymbolState;
use std::collections::BTreeMap;
use wickra_backtest_core::Candle;

/// One symbol's emitted rows: `(row identity, cells)` in bar order.
type SymbolRows = Vec<(RowId, Vec<f64>)>;

/// Build a feature matrix from a symbol -> candles map.
///
/// Columns are `spec.features` (in order) followed by `spec.labels`. Rows are
/// emitted per symbol in sorted order, ascending by bar within each symbol.
///
/// # Errors
/// Returns [`crate::Error::BadSpec`] on an invalid spec and
/// [`crate::Error::UnknownIndicator`] if a referenced indicator is not in the
/// registry.
pub fn build(data: &BTreeMap<String, Vec<Candle>>, spec: &FeatureSpec) -> Result<FeatureMatrix> {
    spec.validate()?;
    // Resolve every indicator once so an unknown name fails even with no data.
    let _ = IndicatorSet::from_features(&spec.features)?;

    let entries: Vec<(&String, &Vec<Candle>)> = data.iter().collect();

    #[cfg(feature = "parallel")]
    let per_symbol: Vec<Result<SymbolRows>> = {
        use rayon::prelude::*;
        entries
            .par_iter()
            .map(|(symbol, candles)| build_symbol(spec, symbol, candles))
            .collect()
    };
    #[cfg(not(feature = "parallel"))]
    let per_symbol: Vec<Result<SymbolRows>> = entries
        .iter()
        .map(|(symbol, candles)| build_symbol(spec, symbol, candles))
        .collect();

    let mut matrix = FeatureMatrix::new(spec.columns());
    for symbol_rows in per_symbol {
        for (id, cells) in symbol_rows? {
            matrix.push_row(id, cells);
        }
    }

    if let Some(scaling) = spec.scaling {
        apply_scaling(&mut matrix, spec.feature_count(), scaling);
    }
    Ok(matrix)
}

/// Build one symbol's rows: fold features bar by bar, join labels from the OHLC
/// arrays, apply the warmup policy and trailing window.
fn build_symbol(spec: &FeatureSpec, symbol: &str, candles: &[Candle]) -> Result<SymbolRows> {
    let m = candles.len();
    let mut state = SymbolState::new(&spec.features)?;
    let mut feature_rows: Vec<Vec<f64>> = Vec::with_capacity(m);
    let mut ready: Vec<bool> = Vec::with_capacity(m);
    for candle in candles {
        state.fold(candle);
        feature_rows.push(state.feature_row());
        ready.push(state.all_ready());
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

    let mut rows: SymbolRows = Vec::new();
    for i in 0..m {
        if spec.warmup == WarmupPolicy::Skip && !ready[i] {
            continue;
        }
        let mut cells = feature_rows[i].clone();
        for label in &spec.labels {
            let value = match label {
                Label::ForwardReturn { horizon, log } => forward_return(&closes, i, *horizon, *log),
                Label::TripleBarrier { horizon, up, down } => {
                    triple_barrier(&highs, &lows, &closes, i, *horizon, *up, *down)
                }
            };
            cells.push(value);
        }
        rows.push((
            RowId {
                symbol: symbol.to_string(),
                ts: candles[i].time,
            },
            cells,
        ));
    }

    if let Some(n) = spec.window {
        if rows.len() > n {
            rows.drain(0..rows.len() - n);
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{Feature, PriceField};
    use crate::spec::Scaling;

    fn candle(t: i64, close: f64) -> Candle {
        Candle {
            time: t,
            open: close,
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume: 10.0,
        }
    }

    fn series(symbol: &str, closes: &[f64]) -> (String, Vec<Candle>) {
        let candles = closes
            .iter()
            .enumerate()
            .map(|(i, &c)| candle(i64::try_from(i).unwrap(), c))
            .collect();
        (symbol.to_string(), candles)
    }

    fn spec_with(features: Vec<Feature>, labels: Vec<Label>) -> FeatureSpec {
        FeatureSpec {
            universe: vec!["AAA".into()],
            timeframe: None,
            features,
            labels,
            window: None,
            output: crate::spec::OutputFormat::Json,
            scaling: None,
            warmup: WarmupPolicy::Nan,
        }
    }

    #[test]
    fn price_and_forward_return_columns() {
        let mut data = BTreeMap::new();
        let (s, c) = series("AAA", &[100.0, 110.0, 121.0]);
        data.insert(s, c);
        let spec = spec_with(
            vec![Feature::Price {
                field: PriceField::Close,
            }],
            vec![Label::ForwardReturn {
                horizon: 1,
                log: false,
            }],
        );
        let m = build(&data, &spec).unwrap();
        assert_eq!(m.columns, vec!["price.close", "fwd_return(1)"]);
        assert_eq!(m.rows, 3);
        // bar 0: close 100, fwd 110/100-1 = 0.1
        assert!((m.data[0][0] - 100.0).abs() < 1e-12);
        assert!((m.data[0][1] - 0.1).abs() < 1e-12);
        // last bar has no future -> NaN label
        assert!(m.data[2][1].is_nan());
    }

    #[test]
    fn warmup_skip_drops_not_ready_rows() {
        let mut data = BTreeMap::new();
        let (s, c) = series("AAA", &[1.0, 2.0, 3.0, 4.0]);
        data.insert(s, c);
        let mut spec = spec_with(
            vec![Feature::Indicator {
                name: "Sma".into(),
                params: vec![3.0],
                field: None,
            }],
            vec![],
        );
        spec.warmup = WarmupPolicy::Skip;
        let m = build(&data, &spec).unwrap();
        // 3-bar SMA ready from bar index 2 onward -> 2 rows.
        assert_eq!(m.rows, 2);
    }

    #[test]
    fn window_keeps_trailing_rows() {
        let mut data = BTreeMap::new();
        let (s, c) = series("AAA", &[1.0, 2.0, 3.0, 4.0, 5.0]);
        data.insert(s, c);
        let mut spec = spec_with(
            vec![Feature::Price {
                field: PriceField::Close,
            }],
            vec![],
        );
        spec.window = Some(2);
        let m = build(&data, &spec).unwrap();
        assert_eq!(m.rows, 2);
        assert!((m.data[0][0] - 4.0).abs() < 1e-12);
        assert!((m.data[1][0] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn symbols_emit_in_sorted_order() {
        let mut data = BTreeMap::new();
        for (s, c) in [series("BBB", &[1.0, 2.0]), series("AAA", &[3.0, 4.0])] {
            data.insert(s, c);
        }
        let spec = spec_with(
            vec![Feature::Price {
                field: PriceField::Close,
            }],
            vec![],
        );
        let m = build(&data, &spec).unwrap();
        assert_eq!(m.index[0].symbol, "AAA");
        assert_eq!(m.index[2].symbol, "BBB");
    }

    #[test]
    fn scaling_applies_to_feature_columns_only() {
        let mut data = BTreeMap::new();
        let (s, c) = series("AAA", &[10.0, 20.0, 30.0]);
        data.insert(s, c);
        let mut spec = spec_with(
            vec![Feature::Price {
                field: PriceField::Close,
            }],
            vec![Label::ForwardReturn {
                horizon: 1,
                log: false,
            }],
        );
        spec.scaling = Some(Scaling::MinMax);
        let m = build(&data, &spec).unwrap();
        // feature min-max over [10,20,30] -> 0, .5, 1
        assert!(m.data[0][0].abs() < 1e-12);
        assert!((m.data[2][0] - 1.0).abs() < 1e-12);
        // label column untouched by scaling (bar 0: 20/10-1 = 1.0)
        assert!((m.data[0][1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn unknown_indicator_errors_even_without_data() {
        let data: BTreeMap<String, Vec<Candle>> = BTreeMap::new();
        let spec = spec_with(
            vec![Feature::Indicator {
                name: "NotReal".into(),
                params: vec![],
                field: None,
            }],
            vec![],
        );
        assert!(build(&data, &spec).is_err());
    }
}
