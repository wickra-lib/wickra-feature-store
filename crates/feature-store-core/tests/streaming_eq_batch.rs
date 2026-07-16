//! The streaming path must equal the batch path byte-for-byte: pushing a
//! universe bar-by-bar into a `FeatureStore` and building yields exactly the
//! matrix `build(&data, &spec)` produces, including the `null` (NaN) label cells
//! at the end of each symbol's history.

mod common;

use std::fs;

use feature_store_core::{build, FeatureStore};

#[test]
fn streaming_matches_batch_for_every_golden_spec() {
    let data = common::load_data();
    let specs_dir = common::golden_dir().join("specs");

    for (name, spec) in common::load_specs() {
        // The batch reference.
        let batch = build(&data, &spec)
            .unwrap_or_else(|e| panic!("batch build {name}: {e}"))
            .to_json();

        // The streaming build: feed every symbol bar-by-bar in sorted order.
        let spec_json = fs::read_to_string(specs_dir.join(format!("{name}.json")))
            .unwrap_or_else(|e| panic!("read spec {name}: {e}"));
        let mut store = FeatureStore::new(&spec_json)
            .unwrap_or_else(|e| panic!("FeatureStore::new {name}: {e}"));
        for (symbol, candles) in &data {
            for candle in candles {
                store.push(symbol, candle);
            }
        }
        let streamed = store
            .build()
            .unwrap_or_else(|e| panic!("streaming build {name}: {e}"))
            .to_json();

        assert_eq!(
            streamed, batch,
            "streaming != batch for {name} — the two build paths must be byte-identical"
        );
    }
}

#[test]
fn reset_clears_the_streamed_state() {
    let data = common::load_data();
    let (name, spec) = common::load_specs().into_iter().next().expect("a spec");
    let spec_json = fs::read_to_string(
        common::golden_dir()
            .join("specs")
            .join(format!("{name}.json")),
    )
    .expect("read spec");

    let mut store = FeatureStore::new(&spec_json).expect("new");
    for (symbol, candles) in &data {
        for candle in candles {
            store.push(symbol, candle);
        }
    }
    let before = store.build().expect("build before reset").to_json();

    store.reset();
    let empty = store.build().expect("build after reset");
    assert_eq!(empty.rows, 0, "reset must drop all accumulated bars");

    // Re-pushing the same data reproduces the pre-reset matrix.
    for (symbol, candles) in &data {
        for candle in candles {
            store.push(symbol, candle);
        }
    }
    let after = store.build().expect("build after re-push").to_json();
    assert_eq!(
        after, before,
        "re-pushing after reset must reproduce the matrix"
    );

    let batch = build(&data, &spec).expect("batch").to_json();
    assert_eq!(before, batch, "streamed build must equal batch build");
}
