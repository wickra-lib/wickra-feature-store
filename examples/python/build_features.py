"""A runnable Python example: build a feature matrix through the binding.

    pip install wickra-feature-store
    python examples/python/build_features.py
"""

import json

from wickra_feature_store import FeatureStore

SPEC = json.dumps(
    {
        "universe": ["AAA", "BBB"],
        "features": [
            {"kind": "indicator", "name": "Sma", "params": [2]},
            {"kind": "price", "field": "close"},
        ],
        "labels": [{"kind": "forward_return", "horizon": 1}],
    }
)


def candle(time: int, close: float) -> dict:
    return {
        "time": time,
        "open": close,
        "high": close,
        "low": close,
        "close": close,
        "volume": 1.0,
    }


def series(closes: list[float]) -> list[dict]:
    return [candle(i + 1, c) for i, c in enumerate(closes)]


def main() -> None:
    store = FeatureStore(SPEC)
    response = store.command(
        json.dumps(
            {
                "cmd": "build_batch",
                "data": {
                    "AAA": series([10.0, 11.0, 12.0]),
                    "BBB": series([20.0, 22.0, 24.0]),
                },
            }
        )
    )
    matrix = json.loads(response)

    print(f"wickra-feature-store {FeatureStore.version()}")
    print(f"columns: {matrix['columns']}")
    print(f"rows: {matrix['rows']}")
    print(response)


if __name__ == "__main__":
    main()
