//! A runnable Rust example: build a feature matrix from a small universe with
//! the native `build` API and print the resulting matrix JSON.
//!
//! ```bash
//! cargo run -p wickra-feature-store-example
//! ```

use std::collections::BTreeMap;

use feature_store_core::{build, Candle, FeatureSpec};

const SPEC: &str = r#"{
    "universe": ["AAA", "BBB"],
    "features": [
        {"kind": "indicator", "name": "Sma", "params": [2]},
        {"kind": "price", "field": "close"}
    ],
    "labels": [{"kind": "forward_return", "horizon": 1}]
}"#;

fn candle(time: i64, close: f64) -> Candle {
    Candle {
        time,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.0,
    }
}

fn series(closes: &[f64]) -> Vec<Candle> {
    closes
        .iter()
        .enumerate()
        .map(|(i, &c)| candle(i64::try_from(i).unwrap() + 1, c))
        .collect()
}

fn main() {
    let spec: FeatureSpec = FeatureSpec::from_json(SPEC).expect("valid spec");

    let mut data = BTreeMap::new();
    data.insert("AAA".to_string(), series(&[10.0, 11.0, 12.0]));
    data.insert("BBB".to_string(), series(&[20.0, 22.0, 24.0]));

    let matrix = build(&data, &spec).expect("build feature matrix");

    println!("wickra-feature-store {}", feature_store_core::version());
    println!("columns: {:?}", matrix.columns);
    println!("rows: {}", matrix.rows);
    println!("{}", matrix.to_json());
}
