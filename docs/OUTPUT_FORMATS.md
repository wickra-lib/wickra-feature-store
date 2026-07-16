# Output formats

A built `FeatureMatrix` can be serialised four ways, chosen by the spec's
`output` field (or the CLI `--format` flag):

| Format | `output` | Availability | Notes |
|--------|----------|--------------|-------|
| JSON | `json` (default) | every language, every target | Canonical, byte-identical across languages. |
| CSV | `csv` | every language, every target | Header `symbol,ts,<col>,…`; `NaN` → empty field. |
| Apache Arrow IPC | `arrow` | native only (`arrow` build feature) | Binary; CLI or native Rust API. |
| Apache Parquet | `parquet` | native only (`arrow` build feature) | Binary; CLI or native Rust API. |

## JSON — the canonical format

```json
{
  "columns": ["Sma(10)", "Ema(10)", "price.close", "fwd_return(5)"],
  "index":   [{"symbol": "sym-01", "ts": 1700000000}],
  "data":    [[101.2, 101.4, 101.0, 0.021]],
  "rows":    1
}
```

- `columns` is `features` (spec order) followed by `labels` (spec order), never
  sorted.
- `index[r]` is the `{symbol, ts}` of row `r`, in emission order.
- `data[r]` has one cell per column. Every finite cell is rounded to `1e-8`
  (`round(x * 1e8) / 1e8`); `NaN` is written as JSON `null`; `±inf` never
  appears (it collapses to `null`). The literals `NaN`/`Infinity` are never
  emitted — they are not valid JSON.

Because each binding returns the core's JSON string **verbatim**, the JSON output
is byte-identical across all ten languages. This is what the cross-language
golden test pins.

## CSV

A flat table: `symbol,ts,<col1>,<col2>,…`. Cells use the same `1e-8` rounding;
`NaN` becomes an empty field. Good for spreadsheets and quick inspection.

## Arrow / Parquet (native only)

Arrow and Parquet are columnar binary formats for large matrices and fast
downstream loading (pandas, polars, DuckDB). The schema is `symbol: Utf8`,
`ts: Int64`, and one `Float64` column per matrix column (`NaN` is represented
natively).

They are produced only by the native Rust API or the CLI, and only when the crate
is built with the `arrow` feature:

```bash
wickra-feature-store --spec spec.json --data ./data --format parquet --out features.parquet
```

They are **not** available over the `command(json)` FFI boundary — the boundary
is JSON text, so a binary payload cannot cross it. WASM builds ship without the
`arrow` feature, so in the browser only `json` and `csv` are available; request
`parquet` there and you get an error telling you to use the CLI or native API.
Arrow/Parquet are also excluded from the byte-for-byte golden comparison — they
are covered by a native round-trip test (`build → write → read → assert equal`)
instead.

## See also

- [STREAMING.md](STREAMING.md) — the command boundary and JSON responses.
- [Cookbook.md](Cookbook.md) — CLI recipes for each format.
