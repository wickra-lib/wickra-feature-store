#![no_main]
//! Fuzz the spec-parsing surface: arbitrary bytes are parsed as a `FeatureSpec`
//! (JSON). Malformed input must surface as a clean `Err`, never a panic. A
//! successfully parsed spec re-serializes and re-parses to an equal value, and
//! `validate` never panics.

use feature_store_core::FeatureSpec;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(spec) = FeatureSpec::from_json(text) else {
        return;
    };
    // Validation is total: it returns Ok/Err, never panics.
    let _ = spec.validate();
    // Column keys never panic on a parsed spec.
    let _ = spec.columns();
    // A parsed spec round-trips: serialize -> parse -> equal.
    let serialized = serde_json::to_string(&spec).expect("serialize a parsed spec");
    let reparsed: FeatureSpec =
        serde_json::from_str(&serialized).expect("re-parse a serialized spec");
    assert_eq!(reparsed, spec, "FeatureSpec serde round-trip is not stable");
});
