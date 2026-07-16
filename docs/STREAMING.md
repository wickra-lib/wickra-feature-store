# Streaming and warmup

A feature store can be driven in two equivalent ways: a one-shot **batch** build
over a full dataset, or an incremental **streaming** build where candles are
pushed one at a time. Both produce the byte-identical matrix for the same total
history — this is asserted by the golden `streaming` test.

## Warmup and row emission

Rows are produced per symbol in bar order `i = 0 .. m-1`, then all symbols are
concatenated in `symbol_key` order (stable blocks). The `warmup` field controls
what happens while indicators are not yet ready:

| `warmup` | Behaviour |
|----------|-----------|
| `nan` (default) | Every bar is emitted; not-yet-ready feature cells and not-yet-computable label cells are `NaN`. |
| `skip` | A row is emitted only once **every feature cell** is non-`NaN`. A `NaN` label never drops a row. |

An optional `window: N` keeps only the last `N` rows per symbol, applied **after**
the warmup policy.

## Command boundary

Every language binding talks to the core through a single `command(json) -> json`
entry point. The commands:

| `cmd` | Fields | Response |
|-------|--------|----------|
| `set_spec` | `spec` | `{"ok":true}` |
| `push` | `symbol`, `candle` | `{"ok":true}` |
| `push_batch` | `symbol`, `candles` | `{"ok":true}` |
| `build` | — (uses streamed state) | the `FeatureMatrix` |
| `build_batch` | `data: {SYMBOL: [candle,...]}` | the `FeatureMatrix` |
| `labels` | — | the matrix with only label columns |
| `reset` | — | `{"ok":true}` (clears candles, keeps spec) |
| `version` | — | `{"version":"0.1.0"}` |

A candle is `{"ts": <i64>, "open", "high", "low", "close", "volume"}`. Any error
(bad spec, unknown indicator, unknown command) comes back in-band as
`{"ok":false,"error":"<message>"}` — never a panic. Binary formats
(`arrow`/`parquet`) do not travel over this boundary; see
[OUTPUT_FORMATS.md](OUTPUT_FORMATS.md).

## Streaming example

```json
{"cmd":"push_batch","symbol":"sym-01","candles":[
  {"ts":1700000000,"open":100,"high":101,"low":99,"close":100.5,"volume":10},
  {"ts":1700003600,"open":100.5,"high":102,"low":100,"close":101.5,"volume":12}]}
{"cmd":"build"}
```

`build` in streaming mode materialises label cells only as far as the forward
horizon has actually been pushed; the remaining look-ahead cells are `NaN`. Feed
the same total history and the streamed `build` reproduces the `build_batch`
matrix exactly.

## See also

- [FEATURES.md](FEATURES.md) / [LABELS.md](LABELS.md) — column definitions.
- [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) — the matrix serialisations.
- [Cookbook.md](Cookbook.md) — CLI and per-language recipes.
