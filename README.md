<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
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

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — deterministic across ten languages.**

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same data-driven core and ten-language binding surface also power [wickra-exchange](https://github.com/wickra-lib/wickra-exchange), [wickra-backtest](https://github.com/wickra-lib/wickra-backtest), [wickra-terminal](https://github.com/wickra-lib/wickra-terminal) and 20 more — see [the full list](https://github.com/wickra-lib).

Wickra Feature Store is one data-driven core, `feature-store-core`: a serde
**`FeatureSpec`** is folded over each symbol's history with the
[Wickra](https://github.com/wickra-lib/wickra) indicator library — 497 of them
resolvable by name — emitting one **feature row per bar**, joining forward-looking
**labels**, optionally scaling, and materializing a **feature matrix** — in
parallel (rayon) or sequentially (the WASM fallback), **byte-for-byte
identical**.

Because the spec is **data, not code**, the exact same feature build crosses the
C ABI and WASM unchanged. The core is exposed as a **JSON-over-C-ABI data API**
(`FeatureStore::command`) in **Rust, Python, Node.js, WASM, C, C++, C#, Go, Java
and R**, so a developer in any language builds the same features.

- **Batch** — `build(data, spec)` folds every symbol over its full history and emits one row per bar.
- **Streaming** — `push(symbol, candle)` + `build()`, O(1) per tick, over the state accumulated so far.
- **Side feeds** — a column whose indicator reads a reference series, a derivatives tick, an order book, the bar's trades or the market cross-section gets it, or the spec is refused by name.

```rust
use feature_store_core::{build, FeatureSpec, SymbolInput};
use std::collections::BTreeMap;

let spec: FeatureSpec = serde_json::from_str(r#"{
    "universe": ["AAA"],
    "features": [{"kind": "indicator", "name": "Rsi", "params": [14]},
                 {"kind": "price", "field": "close"}],
    "labels":   [{"kind": "forward_return", "horizon": 5}]
}"#)?;

let mut data: BTreeMap<String, SymbolInput> = BTreeMap::new();
data.insert("AAA".into(), candles.into());

let matrix = build(&data, &spec)?;
println!("{}", matrix.to_json());
```

## Status

Early development (0.1.0, unreleased). The API is settling ahead of the first
tagged release; the matrix format and command protocol are stable and pinned by
[golden tests](golden/).

## Documentation

- [docs/FEATURES.md](docs/FEATURES.md) — indicator, price and microstructure feature columns.
- [docs/FEEDS.md](docs/FEEDS.md) — the side feeds an indicator reads beyond the candle, and why a spec is refused rather than answered with a dead column.
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
  [docs/FEATURES.md](docs/FEATURES.md). Any of the 497 streaming indicators
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

## Building everything from source

```bash
cargo build --workspace --all-features                 # Rust core + CLI + C ABI
(cd bindings/python && maturin develop --release)      # Python
(cd bindings/node   && npm ci && npm run build)        # Node
(cd bindings/wasm   && wasm-pack build --target web)   # WASM
(cd bindings/csharp && dotnet build)                   # C#
(cd bindings/go     && go build ./...)                 # Go
(cd bindings/java   && mvn -q package)                 # Java
R CMD INSTALL bindings/r                               # R
```

Each binding builds against the C ABI hub in `bindings/c`, so build that first —
`cargo build -p wickra-feature-store-c` — and put the resulting library on the
loader path.

## Testing

```bash
cargo test --workspace --all-features                  # native, Arrow/Parquet on
cargo test --workspace --no-default-features            # sequential / WASM path
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

Every binding runs the same golden corpus from [`golden/`](golden/) and must
produce the identical bytes; that corpus is the cross-language contract, not a
per-language approximation. `python scripts/check_binding_surface.py` asserts the
ten surfaces stayed in step.

## Requirements

- **Rust 1.86+** — the workspace MSRV; the Node binding needs **Rust 1.88**.
- **Python 3.9+** — the Python binding.
- **Node 22+** — the Node binding.
- **Go 1.23+** — the Go binding.
- **Java 22+** — the Java binding.
- **R 2.10+** — the R package.

Per-language toolchains are only needed to build that language's binding — see
its `README.md`.

## Benchmarks

The core cost is folding every symbol's history through its indicators and
computing the labels at each bar. See [BENCHMARKS.md](BENCHMARKS.md) for the
methodology and figures; run them with `cargo bench -p feature-store-bench`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Report vulnerabilities per [SECURITY.md](SECURITY.md).

## Ecosystem

Part of the [Wickra](https://github.com/wickra-lib/wickra) family — each one a
data-driven core with a CLI and the same ten-language binding surface:

- [**wickra**](https://github.com/wickra-lib/wickra) — main library (Rust core + Python / Node.js / WASM bindings + a C ABI for C / C++ / C# / Go / Java / R)
- [**wickra-playground**](https://github.com/wickra-lib/wickra-playground) — a polyglot strategy playground: one StrategySpec live side by side in Python, Rust, JS and Go, entirely in the browser
- [**wickra-exchange**](https://github.com/wickra-lib/wickra-exchange) — unified market-data + execution across ten crypto exchanges
- [**wickra-backtest**](https://github.com/wickra-lib/wickra-backtest) — event-driven backtester over the Wickra core
- [**wickra-terminal**](https://github.com/wickra-lib/wickra-terminal) — the trading terminal: a TUI and a browser renderer over the stack
- [**wickra-xray**](https://github.com/wickra-lib/wickra-xray) — market-microstructure explorer: footprint, order-book heatmap, liquidation map, funding/OI divergence
- [**wickra-radar**](https://github.com/wickra-lib/wickra-radar) — perp-universe alert radar: OI delta, funding flip, book imbalance, liquidation clusters, OI/price divergence
- [**wickra-copilot**](https://github.com/wickra-lib/wickra-copilot) — local market copilot grounded in real order-book, liquidation and funding microstructure
- [**wickra-shazam**](https://github.com/wickra-lib/wickra-shazam) — match an asset's current microstructure fingerprint against its entire history
- [**wickra-benchmark**](https://github.com/wickra-lib/wickra-benchmark) — reproducible, golden-verified benchmark suite — recompute any (strategy, dataset, report) in ten languages and confirm it byte-for-byte
- [**wickra-strategy-ci**](https://github.com/wickra-lib/wickra-strategy-ci) — Jest for trading strategies: golden-pin the report, catch regressions in CI, property-test against fuzzed data
- [**wickra-verify**](https://github.com/wickra-lib/wickra-verify) — confirm or refute a claimed backtest report against its strategy and data, in ten languages
- [**wickra-proof**](https://github.com/wickra-lib/wickra-proof) — Proof-of-Backtest: deterministic (spec, data) → report + blake3 hash, recomputable byte-for-byte in ten languages
- [**wickra-zk**](https://github.com/wickra-lib/wickra-zk) — prove a backtest zero-knowledge — on-chain-verifiable performance without revealing the data or the strategy
- [**wickra-impact**](https://github.com/wickra-lib/wickra-impact) — the backtester that knows you would have moved the market: agent-based fills on the real historical L2 order book
- [**wickra-darwin**](https://github.com/wickra-lib/wickra-darwin) — evolutionary strategy search at hundreds of thousands of backtests per second, mutating and crossing JSON specs across the whole indicator registry
- [**wickra-gym**](https://github.com/wickra-lib/wickra-gym) — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for deterministic RL rollouts
- [**wickra-genome**](https://github.com/wickra-lib/wickra-genome) — a vector database of the whole market: every asset a live vector over the indicator registry, for similarity search, clustering and anomaly detection
- [**wickra-timemachine**](https://github.com/wickra-lib/wickra-timemachine) — scrub the whole market like a video — every symbol, full order book, rewound to any moment via deterministic re-fold
- [**wickra-synth**](https://github.com/wickra-lib/wickra-synth) — deterministic synthetic market microstructure: OHLCV, order book, trades and funding from a single seed
- [**wickra-compile**](https://github.com/wickra-lib/wickra-compile) — compile a strategy spec into a standalone deployable: a WASM module, a self-contained binary, or a `no_std` artifact
- [**wickra-embed**](https://github.com/wickra-lib/wickra-embed) — allocation-free, `no_std` streaming indicators for bare-metal and HFT, byte-for-byte identical to the core
- [**wickra-pico**](https://github.com/wickra-lib/wickra-pico) — the O(1) indicator core running bare-metal on a $5 Raspberry Pi Pico — the LED blinks on the EMA cross

The screener's own guides live in [`docs/`](docs/) beside the code; its site,
with the in-browser demo and the benchmark figures, is at
[screener.wickra.org](https://screener.wickra.org). The indicator library's
reference is at [docs.wickra.org](https://docs.wickra.org) and the org landing
page at [wickra.org](https://wickra.org).

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
