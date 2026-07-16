//! Criterion benchmarks for the Feature Store core.
//!
//! `build` is measured across the cross-product of symbol counts {100, 1000},
//! feature counts {5, 20} and label counts {0, 1}, over a fixed 200-bar history
//! per symbol, so the report captures how the feature fold scales with universe
//! width and column count. The parallel (rayon) and sequential paths are selected
//! at compile time by the `parallel` feature; run with and without
//! `--no-default-features` to compare. A final group measures `to_json`
//! serialization of a materialized matrix.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use feature_store_core::{build, Candle, FeatureSpec};

const BARS: usize = 200;

/// A deterministic, non-degenerate `BARS`-long candle path for one symbol,
/// seeded so different symbols follow different curves.
fn symbol_series(seed: usize) -> Vec<Candle> {
    let phase = seed as f64 * 0.3;
    let closes: Vec<f64> = (0..BARS)
        .map(|i| 100.0 + 10.0 * (i as f64 * 0.1 + phase).sin() + i as f64 * 0.05)
        .collect();
    closes
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let o = if i == 0 { c } else { closes[i - 1] };
            Candle {
                time: 1_700_000_000 + i64::try_from(i).unwrap() * 3600,
                open: o,
                high: o.max(c) + 1.0,
                low: o.min(c) - 1.0,
                close: c,
                volume: 1000.0,
            }
        })
        .collect()
}

fn universe(symbols: usize) -> BTreeMap<String, Vec<Candle>> {
    (0..symbols)
        .map(|s| (format!("SYM{s:05}"), symbol_series(s)))
        .collect()
}

/// A spec with `features` indicator/price columns and `labels` forward-return
/// columns, cycling through a small pool of always-resolving indicators.
fn spec(symbols: usize, features: usize, labels: usize) -> FeatureSpec {
    let mut cols = String::new();
    for f in 0..features {
        if f > 0 {
            cols.push(',');
        }
        let period = 5 + f;
        let feature = match f % 4 {
            0 => format!(r#"{{"kind":"indicator","name":"Sma","params":[{period}]}}"#),
            1 => format!(r#"{{"kind":"indicator","name":"Ema","params":[{period}]}}"#),
            2 => format!(r#"{{"kind":"indicator","name":"Rsi","params":[{period}]}}"#),
            _ => r#"{"kind":"price","field":"close"}"#.to_string(),
        };
        cols.push_str(&feature);
    }
    let mut label_cols = String::new();
    for l in 0..labels {
        if l > 0 {
            label_cols.push(',');
        }
        write!(
            label_cols,
            r#"{{"kind":"forward_return","horizon":{}}}"#,
            l + 1
        )
        .expect("write to string");
    }
    let universe: Vec<String> = (0..symbols).map(|s| format!("\"SYM{s:05}\"")).collect();
    let json = format!(
        r#"{{"universe":[{}],"features":[{cols}],"labels":[{label_cols}]}}"#,
        universe.join(",")
    );
    FeatureSpec::from_json(&json).expect("bench spec parses")
}

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("build");
    for &symbols in &[100usize, 1000] {
        let data = universe(symbols);
        for &features in &[5usize, 20] {
            for &labels in &[0usize, 1] {
                let spec = spec(symbols, features, labels);
                let id = format!("{symbols}sym_{features}feat_{labels}lab");
                group.throughput(Throughput::Elements((symbols * BARS) as u64));
                group.bench_with_input(BenchmarkId::from_parameter(&id), &spec, |b, spec| {
                    b.iter(|| build(&data, spec).expect("build"));
                });
            }
        }
    }
    group.finish();
}

fn bench_serialize(c: &mut Criterion) {
    let data = universe(200);
    let spec = spec(200, 10, 1);
    let matrix = build(&data, &spec).expect("build");
    let mut group = c.benchmark_group("serialize");
    group.throughput(Throughput::Elements(matrix.rows as u64));
    group.bench_function("to_json", |b| b.iter(|| matrix.to_json()));
    group.finish();
}

criterion_group!(benches, bench_build, bench_serialize);
criterion_main!(benches);
