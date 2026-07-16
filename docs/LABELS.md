# Labels

Labels are the target columns of the matrix. They are listed in the spec's
`labels` array and appended, in spec order, after every
[feature](FEATURES.md) column. Labels are look-ahead quantities: the last rows of
each symbol have no future and therefore carry `NaN` (serialised as `null`).
Labels are **never** [scaled](SCALING.md).

There are two label kinds, tagged by `"kind"`:

| Kind | JSON | Column key |
|------|------|-----------|
| `forward_return` | `{"kind":"forward_return","horizon":5}` | `fwd_return(5)` |
| `forward_return` (log) | `{"kind":"forward_return","horizon":10,"log":true}` | `fwd_log_return(10)` |
| `triple_barrier` | `{"kind":"triple_barrier","horizon":20,"up":0.02,"down":0.02}` | `tb(20,0.02,0.02)` |

`horizon` must be greater than zero for every label, or the spec is rejected.

## `forward_return`

The horizon-`h` forward return at bar `i`, for a symbol with bars `c[0..m]`:

```
fwd_return(h)[i]  = close[i+h] / close[i] - 1          # log = false (default)
fwd_log_return(h)[i] = ln(close[i+h] / close[i])       # log = true
```

The value is defined only while `i + h <= m - 1`; the last `h` rows of each
symbol are `NaN`. If `close[i]` is `0.0` the cell is `NaN` rather than an
infinity.

## `triple_barrier`

Lopez de Prado's triple-barrier label, encoded as `+1 / -1 / 0`. With
`entry = close[i]`, walk forward over `k = i+1 .. min(i+h, m-1)` and take the
first barrier touched:

- `high[k] >= entry * (1 + up)` → **+1** (upper barrier hit first)
- `low[k] <= entry * (1 - down)` → **-1** (lower barrier hit first)
- both barriers touched on the **same** `k` → **-1** (the lower barrier wins the
  tie; this rule is fixed and deterministic)
- no barrier touched by `i + h` → **0** (the vertical/time barrier)
- not enough future (`i + h > m - 1`) → **NaN**

No intrabar path is assumed beyond the fixed "down wins the tie" rule, so the
label is fully deterministic.

## Labels versus features

Under `WarmupPolicy::Skip` a row is dropped only when a **feature** cell is
`NaN`; a `NaN` label never drops a row, because a missing target is legitimate —
the consumer decides how to handle it. See [STREAMING.md](STREAMING.md#warmup).

## See also

- [FEATURES.md](FEATURES.md) — the feature columns that precede the labels.
- [SCALING.md](SCALING.md) — why scaling touches features but never labels.
- [../BENCHMARKS.md](../BENCHMARKS.md) — cost of building matrices at scale.
