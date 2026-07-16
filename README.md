<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp" alt="Wickra Feature Store — ML-ready feature matrices over 514 streaming indicators" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-feature-store)
[![CI](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/codeql.yml/badge.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/codeql.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Docs](https://img.shields.io/badge/docs-wickra.org-3b82f6)](https://wickra.org)

---

# Wickra Feature Store

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 514 O(1) streaming indicators — deterministic across 10 languages.**

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same data-driven core and ten-language binding surface also power [wickra-backtest](https://github.com/wickra-lib/wickra-backtest), [wickra-screener](https://github.com/wickra-lib/wickra-screener), [wickra-terminal](https://github.com/wickra-lib/wickra-terminal), [wickra-exchange](https://github.com/wickra-lib/wickra-exchange), [wickra-xray](https://github.com/wickra-lib/wickra-xray), [wickra-radar](https://github.com/wickra-lib/wickra-radar), [wickra-copilot](https://github.com/wickra-lib/wickra-copilot) and [wickra-shazam](https://github.com/wickra-lib/wickra-shazam).

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

- **Batch** — `build(universe, spec)` folds every symbol over its full history and emits one feature row per bar.
- **Streaming** — `push(symbol, candle)` + `build()`, O(1) per tick, for online feature accumulation.
- **Microstructure** — orderflow, funding, open interest and liquidation metrics as first-class feature columns.
- **Output** — JSON and CSV everywhere; Arrow and Parquet on native targets (feature-gated).

## Status

Early development (0.1.0, unreleased). This README is a skeleton; it is finalized
alongside the first release.

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE)
at your option.

## Disclaimer

This software is provided for research and educational purposes. It is not
financial advice.
