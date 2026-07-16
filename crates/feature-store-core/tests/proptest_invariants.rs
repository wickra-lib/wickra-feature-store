//! Property-based invariants over `build`: whatever random universe and spec is
//! fed in (drawn from indicators that always resolve), nothing panics, the
//! matrix shape is exact, the warmup-skip policy never emits a `NaN` feature
//! cell, min-max scaling stays in `[0, 1]`, and the build is deterministic.

use std::collections::BTreeMap;

use feature_store_core::{
    build, Candle, Feature, FeatureSpec, Label, PriceField, Scaling, WarmupPolicy,
};
use proptest::prelude::*;

/// A candle path derived from a base price so OHLC is always finite and ordered.
fn candles_strategy() -> impl Strategy<Value = Vec<Candle>> {
    prop::collection::vec((50.0f64..500.0, 0.0f64..5.0), 5..40).prop_map(|rows| {
        rows.into_iter()
            .enumerate()
            .map(|(i, (close, spread))| Candle {
                time: 1_700_000_000 + i64::try_from(i).unwrap() * 3600,
                open: close,
                high: close + spread + 1.0,
                low: close - spread - 1.0,
                close,
                volume: 1000.0,
            })
            .collect()
    })
}

fn universe_strategy() -> impl Strategy<Value = BTreeMap<String, Vec<Candle>>> {
    prop::collection::vec(candles_strategy(), 1..4).prop_map(|per_symbol| {
        per_symbol
            .into_iter()
            .enumerate()
            .map(|(i, candles)| (format!("SYM{i:02}"), candles))
            .collect()
    })
}

/// Indicators and price fields that always resolve against the registry, so the
/// build never fails on an unknown name — the fuzzed axis is the spec shape.
fn feature_strategy() -> impl Strategy<Value = Feature> {
    prop_oneof![
        (2u32..20).prop_map(|p| Feature::Indicator {
            name: "Sma".into(),
            params: vec![f64::from(p)],
            field: None,
        }),
        (2u32..20).prop_map(|p| Feature::Indicator {
            name: "Ema".into(),
            params: vec![f64::from(p)],
            field: None,
        }),
        (2u32..20).prop_map(|p| Feature::Indicator {
            name: "Rsi".into(),
            params: vec![f64::from(p)],
            field: None,
        }),
        Just(Feature::Price {
            field: PriceField::Close
        }),
        Just(Feature::Price {
            field: PriceField::Volume
        }),
    ]
}

fn label_strategy() -> impl Strategy<Value = Label> {
    prop_oneof![
        (1usize..6).prop_map(|horizon| Label::ForwardReturn {
            horizon,
            log: false
        }),
        (1usize..6).prop_map(|horizon| Label::ForwardReturn { horizon, log: true }),
        (1usize..6).prop_map(|horizon| Label::TripleBarrier {
            horizon,
            up: 0.02,
            down: 0.02,
        }),
    ]
}

prop_compose! {
    fn spec_strategy()(
        features in prop::collection::vec(feature_strategy(), 1..4),
        labels in prop::collection::vec(label_strategy(), 0..3),
        warmup in prop_oneof![Just(WarmupPolicy::Nan), Just(WarmupPolicy::Skip)],
        scaling in prop_oneof![
            Just(None),
            Just(Some(Scaling::ZScore)),
            Just(Some(Scaling::MinMax)),
        ],
        window in prop_oneof![Just(None), (1usize..10).prop_map(Some)],
    ) -> FeatureSpec {
        FeatureSpec {
            universe: vec!["SYM00".into()],
            timeframe: None,
            features,
            labels,
            window,
            output: feature_store_core::OutputFormat::Json,
            scaling,
            warmup,
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn build_never_panics_and_has_exact_shape(
        data in universe_strategy(),
        spec in spec_strategy(),
    ) {
        let matrix = build(&data, &spec).expect("known indicators build cleanly");

        let ncols = spec.features.len() + spec.labels.len();
        prop_assert_eq!(matrix.columns.len(), ncols);
        prop_assert_eq!(matrix.rows, matrix.data.len());
        prop_assert_eq!(matrix.rows, matrix.index.len());
        for row in &matrix.data {
            prop_assert_eq!(row.len(), ncols);
        }
    }

    #[test]
    fn warmup_skip_emits_no_nan_feature_cell(
        data in universe_strategy(),
        mut spec in spec_strategy(),
    ) {
        spec.warmup = WarmupPolicy::Skip;
        let feature_count = spec.features.len();
        let matrix = build(&data, &spec).expect("build");
        for row in &matrix.data {
            for cell in &row[..feature_count] {
                prop_assert!(cell.is_finite(), "skip policy emitted a non-finite feature cell");
            }
        }
    }

    #[test]
    fn min_max_scaling_stays_in_unit_interval(
        data in universe_strategy(),
        mut spec in spec_strategy(),
    ) {
        spec.scaling = Some(Scaling::MinMax);
        let feature_count = spec.features.len();
        let matrix = build(&data, &spec).expect("build");
        for row in &matrix.data {
            for cell in &row[..feature_count] {
                // A constant column scales to 0/0 = NaN; only finite cells are bounded.
                if cell.is_finite() {
                    prop_assert!((-1e-9..=1.0 + 1e-9).contains(cell), "min-max cell {} out of [0,1]", cell);
                }
            }
        }
    }

    #[test]
    fn build_is_deterministic(
        data in universe_strategy(),
        spec in spec_strategy(),
    ) {
        let a = build(&data, &spec).expect("build a").to_json();
        let b = build(&data, &spec).expect("build b").to_json();
        prop_assert_eq!(a, b);
    }
}
