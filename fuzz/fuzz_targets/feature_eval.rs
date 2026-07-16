#![no_main]
//! Fuzz the feature/label column-key surface: arbitrary bytes are parsed as a
//! `Feature` or `Label` (JSON), and their canonical keys are computed. Parsing
//! must never panic, and a parsed value's key must be deterministic and
//! round-trip-stable.

use feature_store_core::{Feature, Label};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    if let Ok(feature) = serde_json::from_str::<Feature>(text) {
        let key = feature.key();
        assert_eq!(key, feature.key(), "Feature::key is not deterministic");
        let reparsed: Feature =
            serde_json::from_str(&serde_json::to_string(&feature).unwrap()).unwrap();
        assert_eq!(reparsed.key(), key, "Feature key not round-trip stable");
    }

    if let Ok(label) = serde_json::from_str::<Label>(text) {
        let key = label.key();
        assert_eq!(key, label.key(), "Label::key is not deterministic");
        // horizon() is total.
        let _ = label.horizon();
        let reparsed: Label =
            serde_json::from_str(&serde_json::to_string(&label).unwrap()).unwrap();
        assert_eq!(reparsed.key(), key, "Label key not round-trip stable");
    }
});
