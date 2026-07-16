# Features

A `FeatureSpec` lists the columns of the output matrix. Feature columns come
first, in spec order, followed by [label](LABELS.md) columns. The column order
is never sorted — it is exactly the order you write it, so a matrix is stable
across languages and across runs.

There are three feature kinds, tagged by `"kind"`:

| Kind | JSON | Column key |
|------|------|-----------|
| `indicator` | `{"kind":"indicator","name":"Rsi","params":[14]}` | `Rsi(14)` |
| `indicator` (sub-output) | `{"kind":"indicator","name":"Macd","params":[12,26,9],"field":"hist"}` | `Macd(12,26,9).hist` |
| `price` | `{"kind":"price","field":"close"}` | `price.close` |
| `microstructure` | `{"kind":"microstructure","metric":"Vwap","params":[]}` | `ms.Vwap()` |

## `indicator`

Any of the 514 streaming indicators from
[`wickra-core`](https://github.com/wickra-lib/wickra). `name` is the indicator's
registry name in PascalCase (`Sma`, `Ema`, `Rsi`, `Atr`, `Macd`, …) and `params`
is its ordered numeric parameter list. Each feature cell is the indicator's
current value after folding the bars `c[0..=i]`; before the indicator is warmed
up the cell is `NaN` (see [warmup](STREAMING.md#warmup)).

Multi-output indicators expose their sub-outputs through the optional `field`.
`Macd` with `"field":"hist"` selects the histogram; omitting `field` (or setting
it to `null`) selects the indicator's primary output. An unknown `name` is a spec
error; an unknown `field` is a spec error.

## `price`

A raw OHLCV column, taken straight from each candle: `open`, `high`, `low`,
`close` or `volume`. Price cells are defined from the first bar (`i >= 0`) — they
never warm up. Prices are modelled as their own feature kind rather than as
pass-through indicators, so they carry no warmup and no parameters.

## `microstructure`

A candle-derived microstructure metric resolved from the same `wickra-core`
registry, addressed by `metric` and `params`. `Vwap` and `Obv` are the
parameter-free examples used by the golden `microstructure` spec; the column key
is `ms.<metric>(<params>)`, e.g. `ms.Vwap()`.

## Worked example

The golden `momentum_features` spec:

```json
{
  "universe": ["sym-01", "sym-02"],
  "features": [
    {"kind": "indicator", "name": "Sma", "params": [10]},
    {"kind": "indicator", "name": "Ema", "params": [10]},
    {"kind": "indicator", "name": "Rsi", "params": [14]},
    {"kind": "indicator", "name": "Macd", "params": [12, 26, 9], "field": "hist"},
    {"kind": "price", "field": "close"}
  ],
  "labels": [{"kind": "forward_return", "horizon": 5}]
}
```

produces columns:

```
["Sma(10)", "Ema(10)", "Rsi(14)", "Macd(12,26,9).hist", "price.close", "fwd_return(5)"]
```

## See also

- [LABELS.md](LABELS.md) — the target columns appended after the features.
- [SCALING.md](SCALING.md) — optional per-column scaling of features.
- [STREAMING.md](STREAMING.md) — how rows are emitted and warmed up.
- [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) — json, csv, arrow and parquet.
