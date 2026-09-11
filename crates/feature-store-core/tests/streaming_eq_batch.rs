//! The streaming path must equal the batch path byte-for-byte: pushing a
//! universe bar-by-bar into a `FeatureStore` and building yields exactly the
//! matrix `build_series(&data, &spec)` produces, including the `null` (NaN)
//! label cells at the end of each symbol's history.
//!
//! The `feeds_*` specs make the same claim about the side feeds: a bar pushed
//! with its reference close, derivatives tick, book, trades and cross-section
//! must fold to the same cells as the parallel arrays a batch build reads.

mod common;

use std::collections::BTreeMap;
use std::fs;

use wickra_feature_store_core::{build_series, FeatureStore, PushFeeds, SymbolSeries};

/// Push a whole symbol series bar by bar, carrying each bar's side feeds.
fn push_series(store: &mut FeatureStore, symbol: &str, series: &SymbolSeries) {
    for (i, candle) in series.candles.iter().enumerate() {
        let feeds = PushFeeds {
            reference: series.reference.as_ref().map(|r| r[i].close),
            deriv: series.derivs.as_ref().map(|d| d[i]),
            orderbook: series.books.as_ref().map(|b| b[i].clone()),
            trades: series.trades.as_ref().map(|t| t[i].clone()),
            cross_section: series.sections.as_ref().map(|s| s[i].clone()),
        };
        if feeds.is_empty() {
            store.push(symbol, candle);
        } else {
            store.push_with(symbol, candle, &feeds);
        }
    }
}

fn push_all(store: &mut FeatureStore, data: &BTreeMap<String, SymbolSeries>) {
    for (symbol, series) in data {
        push_series(store, symbol, series);
    }
}

#[test]
fn streaming_matches_batch_for_every_golden_spec() {
    let specs_dir = common::golden_dir().join("specs");

    for (name, spec) in common::load_specs() {
        let data = common::data_for(&name);

        // The batch reference.
        let batch = build_series(&data, &spec)
            .unwrap_or_else(|e| panic!("batch build {name}: {e}"))
            .to_json();

        // The streaming build: push every symbol bar-by-bar in sorted order,
        // each bar carrying whatever feeds its series holds.
        let spec_json = fs::read_to_string(specs_dir.join(format!("{name}.json")))
            .unwrap_or_else(|e| panic!("read spec {name}: {e}"));
        let mut store = FeatureStore::new(&spec_json)
            .unwrap_or_else(|e| panic!("FeatureStore::new {name}: {e}"));
        push_all(&mut store, &data);
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
    let (name, spec) = common::load_specs().into_iter().next().expect("a spec");
    let data = common::data_for(&name);
    let spec_json = fs::read_to_string(
        common::golden_dir()
            .join("specs")
            .join(format!("{name}.json")),
    )
    .expect("read spec");

    let mut store = FeatureStore::new(&spec_json).expect("new");
    push_all(&mut store, &data);
    let before = store.build().expect("build before reset").to_json();

    store.reset();
    let empty = store.build().expect("build after reset");
    assert_eq!(empty.rows, 0, "reset must drop all accumulated bars");

    // Re-pushing the same data reproduces the pre-reset matrix.
    push_all(&mut store, &data);
    let after = store.build().expect("build after re-push").to_json();
    assert_eq!(
        after, before,
        "re-pushing after reset must reproduce the matrix"
    );

    let batch = build_series(&data, &spec).expect("batch").to_json();
    assert_eq!(before, batch, "streamed build must equal batch build");
}
