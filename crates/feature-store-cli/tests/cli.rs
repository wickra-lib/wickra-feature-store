//! End-to-end tests for the `wickra-feature-store` binary: the CLI's JSON output
//! must be byte-identical to the core build matrix (the determinism moat), and —
//! when built with the `arrow` feature — the columnar path must write a readable
//! file.

use feature_store_core::{build, Candle, FeatureSpec};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const SPEC: &str = r#"{ "universe": ["AAA"],
  "features": [ {"kind":"indicator","name":"Sma","params":[2]}, {"kind":"price","field":"close"} ],
  "labels": [ {"kind":"forward_return","horizon":1} ] }"#;

const CSV: &str = "ts,open,high,low,close,volume\n1,100,101,99,100,10\n2,100,102,99,101,11\n3,101,103,100,102,12\n";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_wickra-feature-store")
}

/// A fresh scratch directory (with a `data/` subfolder) unique to this test.
fn workdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wfs_cli_{}_{tag}", std::process::id()));
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("spec.json"), SPEC).unwrap();
    std::fs::write(dir.join("data").join("AAA.csv"), CSV).unwrap();
    dir
}

/// The universe the CSV fixture decodes to — used to recompute the expected
/// matrix in-process.
fn fixture_universe() -> BTreeMap<String, Vec<Candle>> {
    let candles = vec![
        Candle {
            time: 1,
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.0,
            volume: 10.0,
        },
        Candle {
            time: 2,
            open: 100.0,
            high: 102.0,
            low: 99.0,
            close: 101.0,
            volume: 11.0,
        },
        Candle {
            time: 3,
            open: 101.0,
            high: 103.0,
            low: 100.0,
            close: 102.0,
            volume: 12.0,
        },
    ];
    let mut data = BTreeMap::new();
    data.insert("AAA".to_string(), candles);
    data
}

fn run(dir: &Path, format: &str, extra: &[&str]) -> std::process::Output {
    Command::new(bin())
        .arg("--spec")
        .arg(dir.join("spec.json"))
        .arg("--data")
        .arg(dir.join("data"))
        .args(["--format", format])
        .args(extra)
        .output()
        .expect("spawn CLI")
}

#[test]
fn json_output_equals_the_core_build_matrix() {
    let dir = workdir("json");
    let out = run(&dir, "json", &[]);
    assert!(out.status.success(), "CLI failed: {out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();

    let spec = FeatureSpec::from_json(SPEC).unwrap();
    let expected = build(&fixture_universe(), &spec).unwrap().to_json();
    assert_eq!(stdout.trim_end(), expected);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn csv_output_has_the_column_header() {
    let dir = workdir("csv");
    let out = run(&dir, "csv", &[]);
    assert!(out.status.success(), "CLI failed: {out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("symbol,ts,Sma(2),price.close,fwd_return(1)"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_source_is_an_error() {
    let out = Command::new(bin())
        .args(["--spec", "nope.json"])
        .output()
        .expect("spawn CLI");
    assert!(!out.status.success());
}

#[cfg(feature = "arrow")]
#[test]
fn parquet_output_is_a_readable_file() {
    let dir = workdir("parquet");
    let path = dir.join("out.parquet");
    let out = run(&dir, "parquet", &["--out", path.to_str().unwrap()]);
    assert!(out.status.success(), "CLI failed: {out:?}");

    let bytes = std::fs::read(&path).unwrap();
    // Parquet files are framed by the `PAR1` magic at both ends.
    assert!(bytes.len() > 8);
    assert_eq!(&bytes[..4], b"PAR1");
    assert_eq!(&bytes[bytes.len() - 4..], b"PAR1");

    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(feature = "arrow")]
#[test]
fn parquet_requires_an_output_path() {
    let dir = workdir("parquet_no_out");
    let out = run(&dir, "parquet", &[]);
    assert!(!out.status.success());
    let _ = std::fs::remove_dir_all(&dir);
}
