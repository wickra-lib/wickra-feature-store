//! Serde-boundary conformance: every `Feature`, `Label`, `OutputFormat`,
//! `Scaling` and `WarmupPolicy` variant round-trips through its pinned JSON tag,
//! the column keys are the documented strings, and malformed specs fail
//! definitively rather than silently.

use std::collections::BTreeMap;

use feature_store_core::{
    build, Candle, Error, Feature, FeatureSpec, Label, OutputFormat, PriceField, Scaling,
    WarmupPolicy,
};
use serde_json::json;

/// A value round-trips iff serializing it yields the pinned JSON and parsing that
/// JSON yields an equal value.
fn roundtrip<T>(value: &T, pinned: serde_json::Value)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let serialized = serde_json::to_value(value).expect("serialize");
    assert_eq!(serialized, pinned, "serialized tag drift");
    let parsed: T = serde_json::from_value(pinned).expect("parse");
    assert_eq!(&parsed, value, "round-trip inequality");
}

#[test]
fn feature_tags_are_pinned() {
    roundtrip(
        &Feature::Indicator {
            name: "Rsi".into(),
            params: vec![14.0],
            field: None,
        },
        json!({"kind": "indicator", "name": "Rsi", "params": [14.0], "field": null}),
    );
    roundtrip(
        &Feature::Indicator {
            name: "Macd".into(),
            params: vec![12.0, 26.0, 9.0],
            field: Some("hist".into()),
        },
        json!({"kind": "indicator", "name": "Macd", "params": [12.0, 26.0, 9.0], "field": "hist"}),
    );
    roundtrip(
        &Feature::Price {
            field: PriceField::Close,
        },
        json!({"kind": "price", "field": "close"}),
    );
    roundtrip(
        &Feature::Microstructure {
            metric: "Vwap".into(),
            params: vec![],
        },
        json!({"kind": "microstructure", "metric": "Vwap", "params": []}),
    );
}

#[test]
fn label_tags_are_pinned() {
    roundtrip(
        &Label::ForwardReturn {
            horizon: 5,
            log: false,
        },
        json!({"kind": "forward_return", "horizon": 5, "log": false}),
    );
    roundtrip(
        &Label::ForwardReturn {
            horizon: 3,
            log: true,
        },
        json!({"kind": "forward_return", "horizon": 3, "log": true}),
    );
    roundtrip(
        &Label::TripleBarrier {
            horizon: 20,
            up: 0.02,
            down: 0.02,
        },
        json!({"kind": "triple_barrier", "horizon": 20, "up": 0.02, "down": 0.02}),
    );
}

#[test]
fn price_field_tags_are_pinned() {
    for (field, tag) in [
        (PriceField::Open, "open"),
        (PriceField::High, "high"),
        (PriceField::Low, "low"),
        (PriceField::Close, "close"),
        (PriceField::Volume, "volume"),
    ] {
        roundtrip(&field, json!(tag));
    }
}

#[test]
fn enum_tags_are_pinned() {
    roundtrip(&OutputFormat::Json, json!("json"));
    roundtrip(&OutputFormat::Csv, json!("csv"));
    roundtrip(&OutputFormat::Arrow, json!("arrow"));
    roundtrip(&OutputFormat::Parquet, json!("parquet"));
    roundtrip(&Scaling::ZScore, json!("z_score"));
    roundtrip(&Scaling::MinMax, json!("min_max"));
    roundtrip(&WarmupPolicy::Nan, json!("nan"));
    roundtrip(&WarmupPolicy::Skip, json!("skip"));
}

#[test]
fn column_keys_are_the_documented_strings() {
    assert_eq!(
        Feature::Indicator {
            name: "Sma".into(),
            params: vec![10.0],
            field: None,
        }
        .key(),
        "Sma(10)"
    );
    assert_eq!(
        Feature::Indicator {
            name: "Macd".into(),
            params: vec![12.0, 26.0, 9.0],
            field: Some("hist".into()),
        }
        .key(),
        "Macd(12,26,9).hist"
    );
    assert_eq!(
        Feature::Price {
            field: PriceField::Close
        }
        .key(),
        "price.close"
    );
    assert_eq!(
        Feature::Microstructure {
            metric: "Vwap".into(),
            params: vec![],
        }
        .key(),
        "ms.Vwap()"
    );
    assert_eq!(
        Label::ForwardReturn {
            horizon: 5,
            log: false
        }
        .key(),
        "fwd_return(5)"
    );
    assert_eq!(
        Label::ForwardReturn {
            horizon: 5,
            log: true
        }
        .key(),
        "fwd_log_return(5)"
    );
    assert_eq!(
        Label::TripleBarrier {
            horizon: 20,
            up: 0.02,
            down: 0.02
        }
        .key(),
        "tb(20,0.02,0.02)"
    );
}

fn candle(time: i64, close: f64) -> Candle {
    Candle {
        time,
        open: close,
        high: close + 1.0,
        low: close - 1.0,
        close,
        volume: 1.0,
    }
}

fn single_symbol(closes: &[f64]) -> BTreeMap<String, Vec<Candle>> {
    let candles = closes
        .iter()
        .enumerate()
        .map(|(i, &c)| candle(i64::try_from(i).unwrap(), c))
        .collect();
    BTreeMap::from([("AAA".to_string(), candles)])
}

#[test]
fn unknown_indicator_is_rejected() {
    let spec = FeatureSpec::from_json(
        r#"{"universe":["AAA"],"features":[{"kind":"indicator","name":"NotARealIndicator","params":[]}]}"#,
    )
    .unwrap();
    let err = build(&single_symbol(&[1.0, 2.0]), &spec).unwrap_err();
    assert!(matches!(err, Error::UnknownIndicator(_)), "{err:?}");
}

#[test]
fn empty_universe_is_rejected() {
    let spec =
        FeatureSpec::from_json(r#"{"universe":[],"features":[{"kind":"price","field":"close"}]}"#)
            .unwrap();
    assert!(matches!(spec.validate(), Err(Error::BadSpec(_))));
}

#[test]
fn empty_features_is_rejected() {
    let spec = FeatureSpec::from_json(r#"{"universe":["AAA"],"features":[]}"#).unwrap();
    assert!(matches!(spec.validate(), Err(Error::BadSpec(_))));
}

#[test]
fn zero_label_horizon_is_rejected() {
    let spec = FeatureSpec::from_json(
        r#"{"universe":["AAA"],"features":[{"kind":"price","field":"close"}],"labels":[{"kind":"forward_return","horizon":0}]}"#,
    )
    .unwrap();
    assert!(matches!(spec.validate(), Err(Error::BadSpec(_))));
}

#[test]
fn zero_window_is_rejected() {
    let spec = FeatureSpec::from_json(
        r#"{"universe":["AAA"],"features":[{"kind":"price","field":"close"}],"window":0}"#,
    )
    .unwrap();
    assert!(matches!(spec.validate(), Err(Error::BadSpec(_))));
}

#[test]
fn spec_defaults_are_stable() {
    let spec = FeatureSpec::from_json(
        r#"{"universe":["AAA"],"features":[{"kind":"price","field":"close"}]}"#,
    )
    .unwrap();
    assert_eq!(spec.output, OutputFormat::Json);
    assert_eq!(spec.warmup, WarmupPolicy::Nan);
    assert!(spec.scaling.is_none());
    assert!(spec.window.is_none());
    assert!(spec.labels.is_empty());
}
