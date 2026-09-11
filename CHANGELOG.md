# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **The core crate carried a name the release could not upload.**
  `feature-store-core` is outside the org's crates.io token scope, which
  creates new crates under the `wickra-` prefix only; `cargo publish` on it
  returns 403 at upload while `--dry-run` passes, and because the publish jobs
  run in parallel the release would have landed on PyPI, npm, NuGet, Maven
  Central and the Go mirror without ever reaching crates.io. The core is now
  `wickra-feature-store-core`, the shape of every released sibling. The
  directory keeps its name; only the package and the
  `wickra_feature_store_core` path moved. The same audit ran across the family
  (xray paid for this with its first tag).

- **The napi bump split a crate in two and the build stopped.**
  `napi-derive-backend` 6.1.3 pulls `convert_case` 0.12 while `napi-derive`
  3.6.3 still uses 0.11, and two versions of a crate are two unrelated types --
  so `napi-derive` itself failed to compile, taking the node binding, the clippy
  job and every `cargo build --workspace` row down with it. Held at 6.1.2, which
  is what the screener runs.

- **`cargo-deny` was set to warn about duplicated crates, so it noted that split
  and moved on.** It is an error now. Only four duplicates exist across this
  workspace and each is a crate part-way through a major release reached through
  two ecosystems; they are skipped by name with the reason recorded, so a fifth
  still fails. Verified by putting 6.1.3 back and watching the check fail on
  `convert_case` before the compiler ever ran.

- **`actionlint` failed on five shell constructs the screener had already
  fixed.** `a && b || c` is not if-then-else -- when the publish succeeded but
  the echo failed, the fallback branch ran and reported "already published";
  `local pkg=$(basename …)` and `export PATH="$(cygpath …)"` hide the command's
  exit status behind `local`/`export`; and an asset count taken from `ls` breaks
  on a filename containing a newline. The runner-label config the linter needs
  for `windows-11-arm` was missing too.

- **`js-yaml` was a runtime dependency of the published npm package.** It was
  added to `dependencies` to force the transitive copy off 4.3.0, the version
  osv-scanner flagged -- but `dependencies` is what every consumer of
  `wickra-feature-store` installs, and nothing in the binding imports it. The
  six sibling packages declare no runtime dependency at all. It is gone; the
  same 4.3.2 now arrives dev-only through `@napi-rs/cli`, which is where the
  other six get it.

- **The C++ example now goes through the C++ hull.** It called the C functions
  directly and rebuilt the two-call length protocol by hand -- the very thing
  `wickra_feature_store.hpp` exists to remove -- which left the shipped C++
  surface built by nothing. Verified by running both: the C and C++ examples
  print byte-identical output.

- **Every C++ hull used the include guard `WICKRA_SCREENER_HPP`.** The C headers
  beside them are guarded correctly; only the `.hpp` files shared one name, so
  including two of the family's headers in the same translation unit dropped the
  second silently. Proven by compiling a file that includes two of them and
  names a class from each: `'Env' is not a member of 'wickra'`. All seven now
  compile standalone and together.

- **The hull's usage example could not run.** It showed a spec shaped
  `{"universe":[...]}` and `{"cmd":"scan"}`, the screener's, which this core
  rejects twice over. It now shows this repository's own spec fields and one of
  its own commands.

- **The issue and pull-request templates asked for a `ScanSpec`**, a type this
  repository does not have, so a contributor was asked to attach something that
  does not exist. `GOVERNANCE.md`, `SUPPORT.md` and `CONTRIBUTING.md` carried
  the same substitution, along with the screener's "condition schema" for a core
  that has no conditions.

- **The R `configure` scripts still defined `wkscreen_download`**, the last
  trace of the screener's prefix — the CI-visible half of which already had to
  be fixed once.

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
- `wickra-feature-store-core`: the data-driven core. A serde `FeatureSpec` (indicator /
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
