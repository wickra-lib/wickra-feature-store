<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![r-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](https://github.com/wickra-lib/wickra-feature-store#license)

# Wickra Feature Store — R

---

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — for R. `install.packages("wickrafeaturestore", repos = "https://wickra-lib.r-universe.dev")` — over the C ABI via `.Call`, prebuilt library fetched on install.**

R bindings for the Wickra feature-matrix core over its C ABI hub, via `.Call`.
A store is built from a spec JSON and driven over a JSON boundary, so the result
is byte-identical to every other Wickra Feature Store binding.

## Install

From r-universe:

```r
install.packages("wickrafeaturestore", repos = "https://wickra-lib.r-universe.dev")
```

The package's `configure` downloads the prebuilt C ABI library for this exact
version from the GitHub release and bundles it, so an ordinary install needs
nothing but a C toolchain (Rtools on Windows) for the thin `.Call` glue layer. To
build against a local checkout instead, point it at the header and library with
the environment variables below.

### Building from this repository (contributors)

The C ABI header and shared library are provided out-of-tree through two
environment variables (set by CI / the installer):

```bash
export WKFEATURESTORE_INC=/path/to/bindings/c/include   # the header dir
export WKFEATURESTORE_LIB=/path/to/target/release       # the library dir
R CMD INSTALL bindings/r
Rscript bindings/r/tests/run_tests.R
```

At run time the loader must find the shared library on `LD_LIBRARY_PATH`
(Linux), `DYLD_LIBRARY_PATH` (macOS) or `PATH` (Windows).

## Quick start

```r
library(wickrafeaturestore)

spec <- paste0(
  '{"universe":["AAA"],',
  '"features":[{"kind":"indicator","name":"Sma","params":[2]},{"kind":"price","field":"close"}],',
  '"labels":[{"kind":"forward_return","horizon":1}]}'
)

store <- wkfeaturestore_new(spec)
response <- wkfeaturestore_command(
  store,
  '{"cmd":"build_batch","data":{"AAA":[{"ts":0,"open":100,"high":100,"low":100,"close":100,"volume":1}]}}'
)
cat(response)
```

### Surface

- **`wkfeaturestore_new(spec_json)`** — build a feature store from a spec JSON
  (an external pointer; freed by a finalizer). Raises an R error if the spec is
  invalid.
- **`wkfeaturestore_command(store, cmd_json)`** — run a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `push`, `push_batch`, `build`, `build_batch`, `labels`, `reset`, `version`.
- **`wkfeaturestore_version()`** — the crate version.

Domain errors (a bad command, an unknown command name) come back as an
`{"ok": false, "error": ...}` response, not as an R error. Arrow / Parquet output
is a binary file format and is not available over this JSON surface; use the
`wickra-feature-store` CLI for columnar output.

### Determinism

The response bytes are identical across languages and between the parallel and
sequential build paths, because the whole feature fold lives once in the Rust
core and this binding forwards its JSON verbatim.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of R's native `.Call` interface over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-feature-store/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-feature-store>
- **Docs** (guides, spec reference, cookbook): <https://feature-store.wickra.org>
- **Runnable example:** [`examples/r/`](https://github.com/wickra-lib/wickra-feature-store/tree/main/examples/r)

- The main project: <https://github.com/wickra-lib/wickra-feature-store>
- Documentation: <https://wickra.org>

Wickra Feature Store ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-feature-store/blob/main/SECURITY.md>.

## Disclaimer

`wickra-feature-store` is research and engineering tooling, not financial advice.
A feature matrix describes historical data under the spec you provide; it makes no
claim about the profitability or future performance of any model trained on it.
Trading carries risk; you are responsible for your own decisions.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-feature-store/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-feature-store/blob/main/LICENSE-MIT) at your option.
