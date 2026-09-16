<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/pypi.svg)](https://pypi.org/project/wickra-feature-store/)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](https://github.com/wickra-lib/wickra-feature-store#license)

# Wickra Feature Store — Python

---

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — for Python. `pip install wickra-feature-store` — prebuilt wheels for Linux, macOS and Windows, nothing to compile.**

Python bindings for [wickra-feature-store](https://github.com/wickra-lib/wickra-feature-store),
the data-driven feature-matrix core. Build a `FeatureStore` from a spec JSON,
drive it with command JSONs, and read back feature matrices — the same command
protocol every language binding speaks.

## Install

```bash
pip install wickra-feature-store
```

Pre-built wheels ship for Linux, macOS and Windows — there is nothing to
compile and no C library to track down.

### Building from this repository (contributors)

```sh
maturin develop --release
pytest -q
```

## Quick start

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

### API

| Method | Description |
|--------|-------------|
| `FeatureStore(spec_json)` | Build a feature store from a spec JSON (raises `ValueError` if invalid). |
| `store.command(cmd_json) -> str` | Apply a command JSON, return the response JSON. Commands: `set_spec`, `push`, `push_batch`, `build`, `build_batch`, `labels`, `reset`, `version`. |
| `FeatureStore.version() -> str` | The library version. |

Arrow / Parquet output is a binary file format and is not available over this
JSON surface; use the `wickra-feature-store` CLI for columnar output.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of PyO3, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-feature-store/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-feature-store>
- **Docs** (guides, spec reference, cookbook): <https://feature-store.wickra.org>
- **Runnable example:** [`examples/python/`](https://github.com/wickra-lib/wickra-feature-store/tree/main/examples/python)

Wickra Feature Store ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-feature-store/blob/main/SECURITY.md>.

## Disclaimer

`wickra-feature-store` is research and engineering tooling, not financial advice.
A feature matrix describes historical data under the spec you provide; it makes no
claim about the profitability or future performance of any model trained on it.
Trading carries risk; you are responsible for your own decisions.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-feature-store/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-feature-store/blob/main/LICENSE-MIT) at your option.
