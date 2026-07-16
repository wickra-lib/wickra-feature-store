# Architecture

`wickra-feature-store` is one data-driven core with many thin consumers. A
feature build is a piece of **data** — a serde `FeatureSpec` — that is folded over
each symbol's history with the [Wickra](https://github.com/wickra-lib/wickra)
library of 514 O(1) streaming indicators, emitting one row per bar into a
`FeatureMatrix`. Because the spec is data, not code, the exact same build runs
natively, across the C ABI and in WASM, byte-for-byte identical.

## The layers

```
CONSUMERS   CLI: crates/feature-store-cli    ·   any language via its binding (command JSON)
      ▲ FeatureMatrix JSON / CSV                                ▲
CORE  crates/feature-store-core:  FeatureSpec (JSON) → per-symbol SymbolState fold (O(1)/bar)
                             → feature + label cells → FeatureMatrix → build (rayon) / streaming
      ▼ data-driven JSON-over-C-ABI API in ten languages
BINDINGS  python · node · wasm · c (C-ABI hub) → c / c++ / c# / go / java / r
CORES  wickra-core (indicators) · wickra-data (Candle / CSV)
```

Each binding ships the same surface — a `FeatureStore` handle plus
`command(json) -> json` and `version` — with its own README, tests, a runnable
example, and a completeness guard.

## The core is data-driven

A `FeatureSpec` is a serde struct, never Rust closures: an ordered list of
`Feature` columns (`indicator`, `price`, `microstructure`), a list of `Label`
columns (`forward_return`, `triple_barrier`), plus `scaling`, `warmup`, `window`
and `output`. Closures cannot cross the C ABI or compile to a WASM data boundary;
a serde spec can. So a Python or Go caller sends the same `FeatureSpec` JSON a
Rust caller would, and gets the same `FeatureMatrix` back. See
[docs/FEATURES.md](docs/FEATURES.md) and [docs/LABELS.md](docs/LABELS.md).

## The command boundary

Every consumer talks to the core through a single JSON-in / JSON-out function,
`FeatureStore::command`. The binding does no logic of its own — it forwards the
command string and returns the core's response verbatim. That verbatim
pass-through is what makes the golden corpus a **cross-language** parity corpus:
the same command produces a byte-identical matrix in every language, with no
per-language JSON reformatting. The command table is in
[docs/STREAMING.md](docs/STREAMING.md).

## Two modes, one result type

- **Batch** — `build(data, spec)` folds every symbol over its full history and
  emits one row per bar. Symbols fold independently, so the build runs in parallel
  via rayon (the default `parallel` feature) and sequentially as the WASM fallback
  (`--no-default-features`) — the two paths produce a byte-identical
  `FeatureMatrix`.
- **Streaming** — `push(symbol, candle)` + `build()`, O(1) per tick, for online
  feature accumulation.

Both modes produce the same matrix for the same total history, pinned by the
golden `streaming` test.

## Determinism

The matrix is byte-identical across languages and across the parallel/sequential
paths by construction:

- Symbols are held in a `BTreeMap` and emitted in key order; within a symbol, rows
  are in ascending bar order.
- Columns are `features` (spec order) then `labels` (spec order) — never sorted.
- Parallel folds are symbol-local; the per-symbol row blocks are merged serially
  in key order.
- [Scaling](docs/SCALING.md) reductions run serially in `(symbol, bar)` order, so
  floating-point rounding is identical regardless of the execution path.
- Every finite cell is rounded to `1e-8`; `NaN` serialises to `null`; the literals
  `NaN`/`Infinity` are never emitted.

## Indicators come from the Wickra core

No indicator mathematics lives in this repository. `SymbolState` builds an
`IndicatorSet` that resolves each feature from the `wickra-core` registry by name
and parameters (the same resolver the backtester uses), so the feature store
inherits all 514 indicators and any future additions for free. `price` columns
read straight from the candle; `microstructure` metrics resolve from the same
registry's microstructure namespace.

## Integration with the rest of Wickra

`wickra-feature-store` sits beside the other Wickra consumers over the same core.
It depends on `wickra-core` (indicators) and `wickra-data` (`Candle` + CSV) and
nothing else at runtime. It never places orders and holds no secret material. The
optional `arrow` build feature adds native Apache Arrow / Parquet output; WASM
builds omit it and serve JSON and CSV — see
[docs/OUTPUT_FORMATS.md](docs/OUTPUT_FORMATS.md).
