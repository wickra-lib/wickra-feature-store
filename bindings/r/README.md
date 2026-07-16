# Wickra Feature Store — R

R bindings for the Wickra feature-matrix core over its C ABI hub, via `.Call`.
A store is built from a spec JSON and driven over a JSON boundary, so the result
is byte-identical to every other Wickra Feature Store binding.

## Build & test

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

## Usage

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

## Surface

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

## Determinism

The response bytes are identical across languages and between the parallel and
sequential build paths, because the whole feature fold lives once in the Rust
core and this binding forwards its JSON verbatim.

## See also

- The main project: <https://github.com/wickra-lib/wickra-feature-store>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either MIT or Apache-2.0, at your option.
