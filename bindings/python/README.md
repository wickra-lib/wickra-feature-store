# Wickra Feature Store — Python

Python bindings for [wickra-feature-store](https://github.com/wickra-lib/wickra-feature-store),
the data-driven feature-matrix core. Build a `FeatureStore` from a spec JSON,
drive it with command JSONs, and read back feature matrices — the same command
protocol every language binding speaks.

## Install

```sh
pip install wickra-feature-store
```

## Usage

```python
import json
from wickra_feature_store import FeatureStore

spec = json.dumps({
    "universe": ["AAA"],
    "features": [
        {"kind": "indicator", "name": "Sma", "params": [2]},
        {"kind": "price", "field": "close"},
    ],
    "labels": [{"kind": "forward_return", "horizon": 1}],
})

store = FeatureStore(spec)

def candle(ts, close):
    return {"ts": ts, "open": close, "high": close,
            "low": close, "close": close, "volume": 1.0}

response = store.command(json.dumps({
    "cmd": "build_batch",
    "data": {"AAA": [candle(0, 100.0), candle(1, 110.0), candle(2, 121.0)]},
}))

matrix = json.loads(response)
print(matrix["columns"])  # ['Sma(2)', 'price.close', 'fwd_return(1)']

# Straight into pandas:
#   import pandas as pd
#   df = pd.DataFrame(matrix["data"], columns=matrix["columns"])
```

## API

| Method | Description |
|--------|-------------|
| `FeatureStore(spec_json)` | Build a feature store from a spec JSON (raises `ValueError` if invalid). |
| `store.command(cmd_json) -> str` | Apply a command JSON, return the response JSON. Commands: `set_spec`, `push`, `push_batch`, `build`, `build_batch`, `labels`, `reset`, `version`. |
| `FeatureStore.version() -> str` | The library version. |

Arrow / Parquet output is a binary file format and is not available over this
JSON surface; use the `wickra-feature-store` CLI for columnar output.

## Build from source

```sh
maturin develop --release
pytest -q
```

## License

`MIT OR Apache-2.0`.
