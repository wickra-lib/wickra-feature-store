# Golden fixtures

The canonical, cross-language golden corpus for `wickra-feature-store`. Every
binding (Rust, Python, Node.js, WASM, C, C++, C#, Go, Java, R) folds this same
spec-and-data corpus and must emit a **byte-identical** `FeatureMatrix` JSON. Do
**not** edit any file here by hand — they are all machine-generated and pinned.

## Layout

- **`data/`** — the deterministic candle universe (`sym-01`…`sym-06`, 48 bars
  each). Each `<symbol>.csv` is `ts,open,high,low,close,volume`; the Rust CLI
  reads this directory directly.
- **`data.json`** — the same universe as a single JSON dataset
  (`{"<symbol>": [{"ts":…,"open":…,…}, …]}`). This is what every language
  binding feeds to `build_batch`; it is generated from `data/` and carries
  byte-for-byte the same candle values.
- **`specs/`** — the canonical `FeatureSpec`s (JSON). Each names its universe,
  feature columns, optional label columns, warmup policy, trailing window and
  scaling.
- **`expected/`** — one `FeatureMatrix` JSON per spec, the golden every language
  binding's `build_batch` response must reproduce byte-for-byte.

The build folds **every symbol present in the data map**, in symbol-sorted
order, ascending by bar within each symbol; the `universe` field lists the six
symbols the corpus carries.

## The specs

| Spec                     | Exercises                                               | Rows |
|--------------------------|--------------------------------------------------------|------|
| `momentum_features.json` | indicators (`Sma`, `Ema`, `Rsi`, `Macd.hist`) + price + `forward_return` label | 288 |
| `microstructure.json`    | microstructure metrics (`Vwap`, `Obv`) + price columns | 288 |
| `triple_barrier.json`    | `Sma` + price + `triple_barrier` label (±1 / 0 outcomes)| 288 |
| `scaled.json`            | z-score scaling applied across feature columns          | 288 |
| `streaming.json`         | `nan` warmup policy + a trailing `window` of 5 rows/symbol | 30 |

## Data formula

Each symbol's close is a fixed, reproducible path:

```
close(i) = base + amp * sin(i / k) + drift * i          (i = 0 .. 47)
open(0)  = close(0);   open(i) = close(i-1)
high(i)  = max(open(i), close(i)) + 1
low(i)   = min(open(i), close(i)) - 1
ts(i)    = 1_700_000_000 + i * 3600
volume   = fixed per symbol
```

| symbol | base | amp | k   | drift | volume |
|--------|------|-----|-----|-------|--------|
| sym-01 | 100  | 10  | 4.0 | 0.05  | 1000   |
| sym-02 | 200  | 8   | 6.0 | 0.10  | 1500   |
| sym-03 | 50   | 5   | 3.0 | 0.20  | 800    |
| sym-04 | 150  | 15  | 5.0 | -0.05 | 1200   |
| sym-05 | 75   | 6   | 8.0 | 0.15  | 600    |
| sym-06 | 120  | 12  | 4.5 | 0.02  | 2000   |

## Regenerating (never by hand)

Re-bless every `expected/<name>.json` from the current engine with the CLI. The
`--stdin` path folds `data.json` through the exact same `build()` call as every
binding's `build_batch`, so the output is byte-identical to what the bindings
must emit:

```bash
cargo build -p wickra-feature-store --release
for s in momentum_features microstructure triple_barrier scaled streaming; do
  ./target/release/wickra-feature-store --spec golden/specs/$s.json --stdin \
    < golden/data.json > golden/expected/$s.json
done
```

`data.json` itself is regenerated from `data/` (same candle values); the CLI's
`--data golden/data` path over the CSV directory produces the identical matrix.

The `golden.rs` conformance test re-runs the same corpus through
`feature-store-core` and asserts each `expected/<name>.json` byte-for-byte, so a
diff in CI means the engine's output changed and the goldens must be re-blessed
deliberately.

## Why the bytes are identical everywhere

Every binding returns the core's `command_json` string **verbatim** — there is
no per-language JSON re-encoding, no float reformatting, and `NaN`/`inf` render
as `null` uniformly in the core. The parallel (rayon) build and the sequential
(`--no-default-features`, WASM) build merge per-symbol rows in the same
symbol-sorted order and run the scaling pass serially, so their output is
byte-for-byte identical. Arrow/Parquet output is a binary format and is **not**
part of this golden corpus; it has its own native round-trip test.
