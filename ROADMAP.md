# Roadmap

`wickra-feature-store` is built out in phases, mirroring the proven structure of the
Wickra exchange, backtester and terminal repos. Each phase lands as reviewed,
CI-green pull requests. Status below is updated as phases complete.

## Phases

0. **Scaffold** — workspace, governance, supply-chain config, `.github`
   scaffolding. *In progress.*
1. **`feature-store-core`** — the `FeatureSpec`, the per-symbol
   `SymbolState` fold, the `FeatureMatrix`, and the `build` / `build_batch`
   entry points, with near-total coverage via inline tests.
2. **`feature-store-cli`** — the reference `wickra-feature-store` binary: load a spec and a
   universe directory, run a build, render the matrix as JSON, CSV or Parquet.
3. **Bindings** — the C ABI hub first, then native Python, Node and WASM, then C,
   C++, C#, Go, Java and R over the hub; each exposes the `FeatureStore` handle +
   `command` + `version`, with a completeness guard.
4. **Golden harness** — a fixed deterministic universe and canonical specs whose
   blessed matrices are the byte-exact, cross-language parity corpus.
5. **Test rigor** — conformance, golden, streaming-equals-batch equivalence,
   property tests, fuzz targets and a criterion benchmark suite.
6. **ABI harness + examples** — cbindgen header sync-check and one runnable
   example per language, with a C/C++ CMake harness.
7. **CI/CD** — the full workflow matrix (all languages), OpenSSF Scorecard, Best
   Practices, link check, and the release workflow.
8. **README, badges, docs** — the banner + badge treatment and the docs guides
   (features, labels, scaling, output formats, streaming, cookbook).

## Beyond 1.0

- Richer feature and label kinds as the corpus grows.
- Additional native output formats alongside JSON, CSV and Arrow / Parquet.

## Non-goals

- **Indicator code in this repository.** Indicators come from the `wickra-core`
  registry; the feature-store composes them, it does not reimplement them.
- **Specs as code.** A `FeatureSpec` is serde data, never a Rust closure,
  so it crosses the C ABI and WASM unchanged.
- **A hosted service or stored credentials.** The feature-store runs locally; it holds
  no secret material, opens no network connection, and places no orders.
