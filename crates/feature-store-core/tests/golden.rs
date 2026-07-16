//! The cross-language golden anchor: fold every committed spec over the golden
//! universe and assert `FeatureMatrix::to_json()` matches its pinned
//! `golden/expected/<spec>.json` byte-for-byte. Every language binding runs this
//! same corpus and must produce the identical bytes; this test is the Rust side
//! of that contract.

mod common;

use std::fs;

use feature_store_core::build;

#[test]
fn every_spec_matches_its_golden_bytes() {
    let data = common::load_data();
    let expected_dir = common::golden_dir().join("expected");
    let specs = common::load_specs();
    assert!(!specs.is_empty(), "golden/specs must not be empty");

    for (name, spec) in specs {
        let matrix = build(&data, &spec).unwrap_or_else(|e| panic!("build {name}: {e}"));
        let got = matrix.to_json();
        let path = expected_dir.join(format!("{name}.json"));
        let want =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert_eq!(
            got,
            want.trim(),
            "golden mismatch for {name} (re-bless if the engine changed deliberately)"
        );
    }
}

/// Re-bless the goldens from the current engine. Ignored by default; run with
/// `cargo test -p feature-store-core --test golden -- --ignored bless` to write
/// every `golden/expected/<spec>.json` from the live output.
#[test]
#[ignore = "writes golden fixtures; run explicitly to re-bless"]
fn bless() {
    let data = common::load_data();
    let expected_dir = common::golden_dir().join("expected");
    fs::create_dir_all(&expected_dir).expect("create expected dir");
    for (name, spec) in common::load_specs() {
        let matrix = build(&data, &spec).unwrap_or_else(|e| panic!("build {name}: {e}"));
        let path = expected_dir.join(format!("{name}.json"));
        fs::write(&path, format!("{}\n", matrix.to_json()))
            .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }
}
