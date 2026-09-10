# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Dependabot watched directories that do not exist**, so it reported nothing
  and the silence read as calm. The `pip` ecosystem did not cover
  `/.github/requirements` and the `npm` one did not cover `/examples/node`,
  though both manifests are here.

- **`release.yml` overwrote the binding READMEs before packing.** Three steps
  copied the root README over `bindings/python/README.md` (wheel and sdist) and
  `bindings/node/README.md`. They date from when the bindings had no README of
  their own; they do now, one per registry, and `check_readme_links.py` exists to
  keep their links absolute because a relative link is dead on PyPI and npm. The
  copy threw that away and shipped the root README, whose links are relative by
  design. The remaining relative links in the C, C#, Go and WASM READMEs are
  absolute now.

- **The Python wheel would have shipped without its licence texts.**
  `bindings/python/` carried neither `LICENSE-MIT` nor `LICENSE-APACHE`, so
  maturin had nothing to include, while every crate and the release archive
  carry both.

- **`SECURITY.md` named a support policy for releases that do not exist yet.**
  It promised fixes for "the latest `0.x` release line" where there is no
  released line; it now says plainly that nothing is published and names `0.1.0`
  as the first version that will be.

- **The bench could not measure the sequential path it advertises.** It took the
  core with default features on -- and `default = ["parallel"]` -- so its own
  `parallel` feature was a no-op and `--no-default-features` changed nothing. In
  the feature store the manifest comment even said "default features off
  (inherited from the workspace edge)", which the workspace edge did not do.

- **Three places still carried 514** after the headline count was corrected to
  497: the workspace manifest's own comment, the npm package description and
  the core crate's module docs.

- **The Ecosystem section repeated two claims their own repositories had already
  corrected**: DARWIN at "millions of backtests per second" across "the
  514-indicator space", where its benchmark says hundreds of thousands over the
  registry, and GENOME as "a 514-dim live vector", where the dimension is
  whatever the spec's feature list names.

- **Indicators that read a side feed produced nothing, silently.**
  `IndicatorSet::update` hardcoded the reference series, derivatives tick,
  order book, trades and cross-section to absent, so an indicator needing
  any of them resolved, ticked, and returned nothing every bar. The column
  was `null` for its whole length while a misspelt name failed loudly.
  Measured before the fix: `Microprice`, `FundingRate`,
  `CumulativeVolumeDelta` and `EffectiveSpread` each produced 0 of 288
  finite values and no error.

- **Every example that crosses the JSON boundary sent the wrong field name
  for the bar timestamp.** The wire contract is `ts`; all eight sent
  `time`, which is the native Rust `Candle` field and only the Rust example
  sees it. They failed at run time with `data did not match any variant of
  untagged enum SymbolInputDoc`; no job ran them, so nothing noticed.

- **The C++ hull could not be compiled by its own build.** It requires
  C++17 (`std::string::data()` returning `char*`); `examples/c/CMakeLists.txt`
  asked for C++14. Nothing compiled it, so nothing found out.

- **The microstructure golden fixture pinned nothing it was named for.** It
  blessed `Vwap` and `Obv`, both candle-driven. The advertised order-flow,
  funding and liquidation metrics had no fixture at all.

### Added

- **Side feeds.** `SymbolSeries` carries the batch form (parallel arrays,
  one entry per candle, mirroring `wickra-backtest`'s `RunRequest`) and a
  `push` carries the per-bar form (`StepFeeds`, reused verbatim). The bare
  candle array still works — `SymbolInput` is untagged, so every existing
  spec, fixture and binding payload is unchanged.

- **A feed check that refuses rather than answers.**
  `FeatureSpec::check_feeds` rejects a spec whose column needs a feed the
  build cannot supply, naming the indicator and the feed; a feed whose
  length differs from the candle count is an error rather than a fold that
  runs out part-way.

- **A fed golden corpus.** `golden/data-feeds.json`, generated
  deterministically by `golden/generate_feeds.py`, plus five `feeds_*`
  specs covering one feed family each. Their blessed output carries real
  values, and every binding runs them.

- **Tests for the C binding**, which had none: streaming-equals-batch in C
  and C++, and a golden test holding all ten specs byte-identical from C.

- The blueprint scaffold: `LICENSES/`, `docs/README.md`, the five long-form
  issue templates, the CodeQL config, the actionlint and CodSpeed
  workflows, the five check scripts, a C++ hull, licence copies in every
  published crate and npm package, and dependabot coverage for the fuzz
  workspace and the Go example.

- CI gains `osv`, `links`, `binding-surface`, `semver`, `fuzz-smoke`,
  `examples` and `python-wheel-container-smoke`; the release pipeline gains
  the `gate` and `guard` jobs, provenance over the nupkg, jar and C ABI
  archives, a Maven artifact on the release page, and a Go mirror that
  builds before it publishes.

### Changed

- **The family pins move to the published releases.**
  `wickra-backtest-core` comes from crates.io at 0.1.4 rather than a git
  rev 130 commits behind, and `wickra-core`/`wickra-data` rise from 0.9 to
  1.0, so the tree carries one set of indicator types rather than two that
  share none.

- **The headline count is 497, not 514.** 514 is the indicator catalogue;
  497 is what the registry resolves by name, which is the number a reader
  can actually build a feature column from.

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
