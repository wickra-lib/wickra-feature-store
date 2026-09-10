//! Shared helpers for the integration tests: load the repo-root golden corpus
//! (the canonical specs, the deterministic candle universe, and the fed one).
//!
//! Every test binary includes this whole module and uses a subset of it, so a
//! helper unused here is another binary's, not dead code.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use feature_store_core::{Candle, FeatureSpec, SymbolInput, SymbolInputDoc, SymbolSeries};

/// The repo-root `golden/` directory, resolved from this crate's manifest dir.
#[must_use]
pub fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("golden")
}

/// Load every `golden/specs/*.json` spec as `(stem, FeatureSpec)`, sorted by
/// file stem so the corpus iterates deterministically.
#[must_use]
pub fn load_specs() -> Vec<(String, FeatureSpec)> {
    let dir = golden_dir().join("specs");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("read golden/specs")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    files
        .iter()
        .map(|f| {
            let stem = f
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("spec stem")
                .to_string();
            let content = fs::read_to_string(f).expect("read spec");
            let spec = FeatureSpec::from_json(&content).expect("parse spec");
            (stem, spec)
        })
        .collect()
}

/// Load every `golden/data/<symbol>.csv` into a symbol-keyed input map. The
/// golden universe is candle-only; the fed corpus lives in `golden/feeds`.
#[must_use]
pub fn load_data() -> BTreeMap<String, SymbolInput> {
    let dir = golden_dir().join("data");
    let mut data = BTreeMap::new();
    for entry in fs::read_dir(&dir).expect("read golden/data") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }
        let symbol = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("csv stem")
            .to_string();
        let content = fs::read_to_string(&path).expect("read csv");
        data.insert(symbol, parse_csv(&content).into());
    }
    data
}

/// Load `golden/data-feeds.json`: the same six symbols and the same candles as
/// `data.json`, each carrying one side feed per family. A spec named `feeds_*`
/// is folded over this dataset instead of the candle-only one.
#[must_use]
pub fn load_fed_data() -> BTreeMap<String, SymbolSeries> {
    let path = golden_dir().join("data-feeds.json");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let raw: BTreeMap<String, SymbolInputDoc> =
        serde_json::from_str(&content).expect("parse data-feeds.json");
    raw.into_iter().map(|(k, v)| (k, v.into())).collect()
}

/// The dataset a spec is folded over: the fed corpus for a `feeds_*` spec, the
/// candle-only one otherwise.
#[must_use]
pub fn data_for(name: &str) -> BTreeMap<String, SymbolSeries> {
    if name.starts_with("feeds_") {
        load_fed_data()
    } else {
        load_data()
            .into_iter()
            .map(|(k, v)| (k, v.to_series()))
            .collect()
    }
}

/// Parse `ts,open,high,low,close,volume` rows; a non-numeric first row is a
/// header and is skipped.
fn parse_csv(content: &str) -> Vec<Candle> {
    let mut candles = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        let time = match cols[0].parse::<i64>() {
            Ok(t) => t,
            Err(_) if idx == 0 => continue, // header row
            Err(e) => panic!("bad ts on line {}: {e}", idx + 1),
        };
        let field = |i: usize| cols[i].parse::<f64>().expect("numeric field");
        candles.push(Candle {
            time,
            open: field(1),
            high: field(2),
            low: field(3),
            close: field(4),
            volume: field(5),
        });
    }
    candles
}
