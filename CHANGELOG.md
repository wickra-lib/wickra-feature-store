# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Repository scaffolding: Cargo workspace, supply-chain configuration
  (`deny.toml`, `osv-scanner.toml`, `lychee.toml`), lint configuration
  (`clippy.toml`), `repo-metadata.toml`, governance docs, the `.github` tree
  (issue/PR templates, `setup-rust`, `sync-metadata.py`, dependabot), and dual
  `MIT OR Apache-2.0` licensing.
- `feature-store-core`: the data-driven core. A serde `FeatureSpec` (indicator /
  price / microstructure features, forward-return / triple-barrier labels,
  optional z-score / min-max scaling, warmup and trailing-window policies) is
  folded over each symbol's history into a deterministic `FeatureMatrix`, in
  parallel (rayon) or sequentially (the WASM path), byte-for-byte identical.
  Output as JSON, CSV, and — with the `arrow` feature — Apache Arrow / Parquet.
- `wickra-feature-store` reference CLI: build a matrix from a directory of CSVs
  or a JSON dataset on stdin, in any output format.
- Ten-language binding surface over a JSON-over-C-ABI hub: Rust, Python,
  Node.js and WASM natively, plus C, C++, C#, Go, Java and R, each with a
  `FeatureStore` handle exposing `command(json) -> json` and `version`.
- Golden corpus (`golden/`) pinning canonical specs to expected matrices, shared
  as a byte-for-byte cross-language parity suite.
- Runnable examples for every language, criterion benchmarks, fuzz targets, and
  the CI, nightly-benchmark and tagged-release workflows.
- Documentation: `docs/FEATURES.md`, `LABELS.md`, `SCALING.md`, `STREAMING.md`,
  `OUTPUT_FORMATS.md` and `Cookbook.md`.

[Unreleased]: https://github.com/wickra-lib/wickra-feature-store/commits/main
