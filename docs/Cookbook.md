# Cookbook

Task-oriented recipes. See [FEATURES.md](FEATURES.md), [LABELS.md](LABELS.md),
[SCALING.md](SCALING.md) for the `FeatureSpec` schema, [STREAMING.md](STREAMING.md)
for the command boundary, and [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) for the
serialisations. Runnable programs live under [`examples/`](../examples).

## Build a matrix from a directory of CSVs

Each symbol is one `<SYMBOL>.csv` file with header `ts,open,high,low,close,volume`.

```bash
# JSON to stdout (the spec's output field, or --format, decides the format).
wickra-feature-store --spec spec.json --data ./data

# CSV instead.
wickra-feature-store --spec spec.json --data ./data --format csv

# Trim to the last 200 rows per symbol.
wickra-feature-store --spec spec.json --data ./data --window 200
```

## Build from a JSON dataset on stdin

```bash
echo '{"sym-01":[{"ts":1700000000,"open":100,"high":101,"low":99,"close":100.5,"volume":10}]}' \
  | wickra-feature-store --spec spec.json --stdin
```

## Write Parquet for a Python/DuckDB pipeline

Binary formats require the `arrow` build feature and an `--out` path:

```bash
wickra-feature-store --spec spec.json --data ./data --format parquet --out features.parquet
```

```python
import duckdb
duckdb.sql("SELECT * FROM 'features.parquet' LIMIT 5").show()
```

## Build in any language

Every binding exposes the same `new(spec) → command(json) → json` surface. The
runnable examples:

- Rust — [`examples/rust`](../examples/rust)
- Python — [`examples/python`](../examples/python)
- Node.js — [`examples/node`](../examples/node)
- C / C++ — [`examples/c`](../examples/c)
- Go — [`examples/go`](../examples/go)
- C# — [`examples/csharp`](../examples/csharp)
- Java — [`examples/java`](../examples/java)
- R — [`examples/r`](../examples/r)

Each binding's own `README.md` (under `bindings/<lang>/`) has install and API
details.

## Stream candles incrementally

```json
{"cmd":"set_spec","spec":{"universe":["sym-01"],"features":[{"kind":"price","field":"close"}]}}
{"cmd":"push","symbol":"sym-01","candle":{"ts":1700000000,"open":100,"high":101,"low":99,"close":100.5,"volume":10}}
{"cmd":"build"}
```

The streamed `build` reproduces the batch matrix for the same total history —
push order within a symbol is chronological.

## Re-bless the golden fixtures

Goldens are produced by the core, never hand-edited. When you deliberately change
behaviour, regenerate them:

```bash
cargo test -p feature-store-core golden -- --ignored --nocapture
git add golden/ && git commit -m "bless feature-store goldens"
```

A golden diff on CI afterwards means the matrix changed — re-bless deliberately,
or fix the regression.

## See also

- [../README.md](../README.md) — overview and quickstart.
- [../ARCHITECTURE.md](../ARCHITECTURE.md) — how the crate is laid out.
- [../BENCHMARKS.md](../BENCHMARKS.md) — throughput at scale.
