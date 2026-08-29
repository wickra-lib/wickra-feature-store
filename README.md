<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 514 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-feature-store)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codeql.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/codeql.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/release.svg)](https://github.com/wickra-lib/wickra-feature-store/releases/latest)
[![crates.io](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/crates.svg)](https://crates.io/crates/wickra-feature-store)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/pypi.svg)](https://pypi.org/project/wickra-feature-store/)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/npm.svg)](https://www.npmjs.com/package/wickra-feature-store)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/nuget.svg)](https://www.nuget.org/packages/Wickra.FeatureStore)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-feature-store)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-feature-store-go)
[![R-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-feature-store)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/best-practices.svg)](https://www.bestpractices.dev)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/provenance.svg)](https://github.com/wickra-lib/wickra-feature-store/attestations)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/docs.svg)](https://wickra.org)
[![Verified across 10 languages](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/verified.svg)](golden/)

---

# Wickra Feature Store

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 514 O(1) streaming indicators — deterministic across ten languages.**

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same data-driven core and ten-language binding surface also power [wickra-exchange](https://github.com/wickra-lib/wickra-exchange), [wickra-backtest](https://github.com/wickra-lib/wickra-backtest), [wickra-terminal](https://github.com/wickra-lib/wickra-terminal) and 20 more — see [the full list](https://github.com/wickra-lib).

Wickra Feature Store is one data-driven core, `feature-store-core`: a serde
**`FeatureSpec`** is folded over each symbol's history with the
[Wickra](https://github.com/wickra-lib/wickra) library of 514 O(1) streaming
indicators, emitting one **feature row per bar**, joining forward-looking
**labels**, optionally scaling, and materializing a **feature matrix** — in
parallel (rayon) or sequentially (the WASM fallback), **byte-for-byte
identical**.

Because the spec is **data, not code**, the exact same feature build crosses the
C ABI and WASM unchanged. The core is exposed as a **JSON-over-C-ABI data API**
(`FeatureStore::command`) in **Rust, Python, Node.js, WASM, C, C++, C#, Go, Java
and R**, so a developer in any language builds the same features.

## Status

Early development (0.1.0, unreleased). The API is settling ahead of the first
tagged release; the matrix format and command protocol are stable and pinned by
[golden tests](golden/).

## Documentation

- [docs/FEATURES.md](docs/FEATURES.md) — indicator, price and microstructure feature columns.
- [docs/LABELS.md](docs/LABELS.md) — forward-return and triple-barrier targets.
- [docs/SCALING.md](docs/SCALING.md) — optional z-score / min-max column scaling.
- [docs/STREAMING.md](docs/STREAMING.md) — the command boundary, warmup and row emission.
- [docs/OUTPUT_FORMATS.md](docs/OUTPUT_FORMATS.md) — JSON, CSV, Arrow and Parquet.
- [docs/Cookbook.md](docs/Cookbook.md) — task-oriented recipes.
- [ARCHITECTURE.md](ARCHITECTURE.md) — how the crate is laid out.
- [BENCHMARKS.md](BENCHMARKS.md) — throughput at scale.

## Quickstart

Build a matrix from a directory of per-symbol CSV files (`<SYMBOL>.csv`, header
`ts,open,high,low,close,volume`):

```bash
wickra-feature-store --spec spec.json --data ./data            # JSON to stdout
wickra-feature-store --spec spec.json --data ./data --format csv
wickra-feature-store --spec spec.json --data ./data --format parquet --out features.parquet
```

A spec lists the feature columns (in order) and the label columns:

```json
{
  "universe": ["sym-01", "sym-02"],
  "features": [
    {"kind": "indicator", "name": "Sma", "params": [10]},
    {"kind": "indicator", "name": "Rsi", "params": [14]},
    {"kind": "indicator", "name": "Macd", "params": [12, 26, 9], "field": "hist"},
    {"kind": "price", "field": "close"}
  ],
  "labels": [{"kind": "forward_return", "horizon": 5}]
}
```

produces columns `["Sma(10)", "Rsi(14)", "Macd(12,26,9).hist", "price.close", "fwd_return(5)"]`.

## Features and labels

- **Features** are `indicator`, `price` or `microstructure` columns — see
  [docs/FEATURES.md](docs/FEATURES.md). Any of Wickra's 514 streaming indicators
  is available by its registry name (`Sma`, `Ema`, `Rsi`, `Macd`, …).
- **Labels** are `forward_return` (arithmetic or log) or `triple_barrier`
  (`+1 / -1 / 0`) — see [docs/LABELS.md](docs/LABELS.md). Look-ahead cells with no
  future are `NaN`.

## Scaling and output formats

Optional per-column [scaling](docs/SCALING.md) (`z_score` or `min_max`) applies to
feature columns only. Matrices serialise as [JSON, CSV, Arrow or
Parquet](docs/OUTPUT_FORMATS.md); JSON and CSV are available in every language and
target, Arrow and Parquet on native builds with the `arrow` feature.

## Use in any language

The core is exposed as a JSON-over-C-ABI data API in ten languages: Rust, Python,
Node.js and WASM natively, plus C, C++, C#, Go, Java and R over the C ABI hub. A
`FeatureStore` handle plus `command(json) -> json` and `version` is the whole
surface; the same spec and data produce a byte-identical matrix in every binding.

```bash
cargo add wickra-feature-store           # Rust
pip install wickra-feature-store         # Python
npm install wickra-feature-store         # Node.js
dotnet add package Wickra.FeatureStore   # C#
go get github.com/wickra-lib/wickra-feature-store/bindings/go   # Go
```

Java ships to Maven Central (`org.wickra:wickra-feature-store`), R to r-universe
(`wickrafeaturestore`), and the C ABI ships as a per-platform library with a
vendored header. Runnable programs for every language live under
[`examples/`](examples/); each binding's `README.md` under [`bindings/`](bindings/)
has install and API details.

## Project layout

```
crates/feature-store-core   # the data-driven core: spec, fold, matrix, formats
crates/feature-store-cli    # wickra-feature-store reference CLI
crates/feature-store-bench  # criterion benchmarks
bindings/{c,python,node,wasm,go,csharp,java,r}   # ten-language surface over the C ABI hub
examples/                   # runnable programs per language
golden/                     # blessed specs + expected matrices (cross-language pin)
docs/                       # deep-dive documentation
```

## Building from source

```bash
cargo test --workspace --all-features                 # native, Arrow/Parquet on
cargo test --workspace --no-default-features           # sequential / WASM path
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Requirements

Rust 1.86+ (workspace MSRV; the Node binding needs 1.88). Per-language toolchains
are only needed to build that language's binding — see its `README.md`.

## Benchmarks

The core cost is folding every symbol's history through its indicators and
computing the labels at each bar. See [BENCHMARKS.md](BENCHMARKS.md) for the
methodology and figures; run them with `cargo bench -p feature-store-bench`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Report vulnerabilities per [SECURITY.md](SECURITY.md).

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

## Disclaimer

`wickra-feature-store` is research and engineering tooling, not financial advice.
A feature matrix describes historical data under the spec you provide; it makes no
claim about the profitability or future performance of any model trained on it.
Trading carries risk; you are responsible for your own decisions.

---

<p align="center">
  <a href="https://github.com/wickra-lib/wickra-feature-store">
    <img alt="GitHub stars" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/stars.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-feature-store/network/members">
    <img alt="GitHub forks" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/forks.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-feature-store/issues">
    <img alt="GitHub issues" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/issues.svg">
  </a>
</p>

<p align="center">
  Built on <a href="https://github.com/wickra-lib/wickra">Wickra</a>. If it saved you time, the cheapest way to say thanks is to ⭐ the repo.
</p>

<p align="center">
  <img alt="wickra-feature-store star history" width="640"
       src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/star-history.svg">
</p>
