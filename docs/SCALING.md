# Scaling

Scaling is optional per-column normalisation of the [feature](FEATURES.md)
columns. It is enabled by the spec's `scaling` field and applies to **feature
columns only** — [label](LABELS.md) columns are never scaled, because a scaled
target would leak the column's global statistics into the label.

```json
{ "features": [ ... ], "scaling": "z_score" }
```

| `scaling` | Formula | Degenerate case | Range |
|-----------|---------|-----------------|-------|
| `z_score` | `(x - mean) / std_pop` | `std_pop == 0` → `0` | ℝ |
| `min_max` | `(x - min) / (max - min)` | `max == min` → `0` | `[0, 1]` |

`std_pop` is the population standard deviation, `sqrt(Σ(x - mean)² / n)`.

## Scope and order

Each feature column is scaled **universe-wide**: the statistics (`mean`, `min`,
`max`, `Σ`) are computed over every emitted, non-`NaN` cell of that column across
all symbols, not per symbol. `NaN` cells stay `NaN` and are excluded from `n`.

The reductions are computed **serially in `(symbol_key, i)` order** — the same
order the rows are emitted — rather than in parallel-fold order. This matters:
floating-point addition is not associative, so a fixed reduction order is what
makes the scaled matrix byte-identical between the `parallel` (rayon) and
sequential builds, and therefore across every language binding.

## When to use it

- Reach for `z_score` when a downstream model assumes roughly zero-mean,
  unit-variance inputs (most linear models, neural nets).
- Reach for `min_max` when you need bounded `[0, 1]` inputs (some tree ensembles,
  distance-based methods).
- Leave `scaling` unset when you want raw indicator values, or when you scale in
  your own pipeline (e.g. a rolling scaler fitted only on training data — the
  universe-wide scaler here uses the whole dataset and can leak test statistics
  into training, so prefer your own fit/transform split for strict backtests).

## See also

- [FEATURES.md](FEATURES.md) — the columns scaling applies to.
- [LABELS.md](LABELS.md) — why labels are excluded.
- [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) — how scaled cells are serialised.
