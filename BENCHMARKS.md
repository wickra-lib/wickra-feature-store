# Benchmarks

A feature store's cost is dominated by folding every symbol's history through its
indicators and computing the label cells at each bar. The benchmarks here measure
that **core build work**, so throughput scales predictably with the universe size
and the number of features a spec references.

## What is measured

The `feature-store-bench` crate (criterion) covers two groups:

- **`build`** — the batch `build(data, spec)` entry point, across a matrix of
  universe size (100 and 1 000 symbols, each carrying 200 bars), feature count
  (5 and 20 indicators), and whether a `forward_return` label is attached.
- **`serialize`** — `FeatureMatrix::to_json` for a 200-symbol, 10-feature,
  1-label matrix, isolating the serialisation cost from the fold.

## Methodology

Run against fixed, in-process synthetic universes so the numbers are reproducible
and contain no I/O variance:

```bash
cargo bench -p feature-store-bench
```

## Results

Median estimates from `cargo bench -p feature-store-bench` on a Windows x86-64
laptop, default `parallel` (rayon) path, with a shortened criterion sampling
window (`--warm-up-time 1 --measurement-time 2`). Treat them as orders of
magnitude, not guarantees — they vary with CPU core count and toolchain.

| Benchmark | Universe × features | Bars folded | Median |
|-----------|---------------------|-------------|--------|
| `build/100sym_5feat_0lab`    | 100 × 5              | 20 000  | 9.9 ms  |
| `build/100sym_5feat_1lab`    | 100 × 5 + label      | 20 000  | 13.0 ms |
| `build/100sym_20feat_0lab`   | 100 × 20             | 20 000  | 37.3 ms |
| `build/100sym_20feat_1lab`   | 100 × 20 + label     | 20 000  | 38.2 ms |
| `build/1000sym_5feat_0lab`   | 1 000 × 5            | 200 000 | 99.0 ms |
| `build/1000sym_5feat_1lab`   | 1 000 × 5 + label    | 200 000 | 126.5 ms |
| `build/1000sym_20feat_0lab`  | 1 000 × 20           | 200 000 | 322 ms  |
| `build/1000sym_20feat_1lab`  | 1 000 × 20 + label   | 200 000 | 290 ms  |
| `serialize/to_json`          | 200 × 10 + label     | —       | 118 ms  |

The takeaway: per-symbol cost stays roughly constant as the universe grows
(~99 µs/symbol at 5 features, ~330 µs/symbol at 20), so build time scales linearly
with universe size and with the number of distinct indicators a spec references — a
1 000-symbol, 5-feature build finishes in ~100 ms. Adding a `forward_return` label
is a small, near-constant overhead. The nightly `bench.yml` workflow reruns the
full criterion sampling on a clean Linux runner for tracking over time.

## Caveats

These figures bound the feature store's own build overhead only. End-to-end time
in a real run also depends on loading the universe from disk or a live feed, which
these in-process benchmarks do not capture.
