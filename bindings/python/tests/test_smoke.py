"""Smoke test: construct a feature store, build a matrix, parse the response."""

import json

from wickra_feature_store import FeatureStore, __version__

SPEC = json.dumps(
    {
        "universe": ["AAA"],
        "features": [
            {"kind": "indicator", "name": "Sma", "params": [2]},
            {"kind": "price", "field": "close"},
        ],
        "labels": [{"kind": "forward_return", "horizon": 1}],
    }
)


def _candle(ts: int, close: float) -> dict:
    return {
        "ts": ts,
        "open": close,
        "high": close,
        "low": close,
        "close": close,
        "volume": 1.0,
    }


def test_build_batch_roundtrip() -> None:
    store = FeatureStore(SPEC)
    response = store.command(
        json.dumps(
            {
                "cmd": "build_batch",
                "data": {"AAA": [_candle(0, 100.0), _candle(1, 110.0), _candle(2, 121.0)]},
            }
        )
    )
    matrix = json.loads(response)
    assert matrix["columns"] == ["Sma(2)", "price.close", "fwd_return(1)"]
    assert matrix["rows"] == 3
    # The optional pandas view: pandas.DataFrame(matrix["data"], columns=matrix["columns"]).


def test_streaming_matches_batch() -> None:
    store = FeatureStore(SPEC)
    for candle in (_candle(0, 100.0), _candle(1, 110.0), _candle(2, 121.0)):
        store.command(json.dumps({"cmd": "push", "symbol": "AAA", "candle": candle}))
    streamed = store.command(json.dumps({"cmd": "build"}))

    batch = FeatureStore(SPEC).command(
        json.dumps(
            {
                "cmd": "build_batch",
                "data": {"AAA": [_candle(0, 100.0), _candle(1, 110.0), _candle(2, 121.0)]},
            }
        )
    )
    assert streamed == batch


def test_version_matches_module() -> None:
    assert FeatureStore.version() == __version__


def test_bad_spec_raises() -> None:
    try:
        FeatureStore("not json")
    except ValueError:
        return
    raise AssertionError("expected ValueError for a malformed spec")
