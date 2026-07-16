#![no_main]
//! Fuzz the numeric fold: arbitrary bytes become a candle path that is folded
//! through a fixed multi-feature, multi-label spec. Whatever the prices, the
//! build must not panic and the matrix shape stays exact.

use std::collections::BTreeMap;

use feature_store_core::{build, Candle, FeatureSpec};
use libfuzzer_sys::fuzz_target;

const SPEC: &str = r#"{
    "universe": ["AAA"],
    "features": [
        {"kind":"indicator","name":"Sma","params":[5]},
        {"kind":"indicator","name":"Rsi","params":[14]},
        {"kind":"price","field":"close"}
    ],
    "labels": [
        {"kind":"forward_return","horizon":3},
        {"kind":"triple_barrier","horizon":5,"up":0.02,"down":0.02}
    ]
}"#;

fuzz_target!(|data: &[u8]| {
    // Up to 128 candles, one per 8 bytes; only finite, bounded prices.
    let mut candles: Vec<Candle> = Vec::new();
    for (i, chunk) in data.chunks_exact(8).take(128).enumerate() {
        let bits = u64::from_le_bytes(chunk.try_into().unwrap());
        let raw = f64::from_bits(bits);
        if !raw.is_finite() {
            continue;
        }
        // Map into a sane positive price band so OHLC stays ordered.
        let close = 1.0 + raw.abs() % 10_000.0;
        candles.push(Candle {
            time: 1_700_000_000 + i64::try_from(i).unwrap() * 3600,
            open: close,
            high: close + 1.0,
            low: (close - 1.0).max(0.01),
            close,
            volume: 1000.0,
        });
    }
    if candles.is_empty() {
        return;
    }

    let spec = FeatureSpec::from_json(SPEC).expect("fixed spec parses");
    let data = BTreeMap::from([("AAA".to_string(), candles)]);
    let matrix = build(&data, &spec).expect("known indicators build cleanly");

    let ncols = spec.features.len() + spec.labels.len();
    assert_eq!(matrix.columns.len(), ncols);
    assert_eq!(matrix.rows, matrix.data.len());
    for row in &matrix.data {
        assert_eq!(row.len(), ncols);
    }
});
