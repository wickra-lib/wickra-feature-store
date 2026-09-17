<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-feature-store/bindings/go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](https://github.com/wickra-lib/wickra-feature-store#license)

# Wickra Feature Store — Go

---

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — for Go. `go get github.com/wickra-lib/wickra-feature-store/bindings/go` — over the C ABI via cgo, prebuilt library bundled in the module.**

Go bindings for the Wickra feature-matrix core over its C ABI hub via cgo. A
`FeatureStore` is built from a spec JSON and driven over a JSON boundary, so the
result is byte-identical to every other Wickra Feature Store binding.

## Install

Use the published **`wickra-feature-store-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps
(a C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-feature-store-go
```

`wickra-feature-store-go` is generated from this directory by the release pipeline: it mirrors
the Go sources, the vendored C ABI header (`include/wickra_feature_store.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Linux/macOS the library path is baked
in via rpath; on Windows the DLL must be discoverable at run time (next to the
executable or on `PATH`).

The prebuilt C ABI library is staged per platform under `lib/<goos>_<goarch>/`
and the header is vendored under `include/`. For a local build, copy the library
built by `cargo build -p wickra-feature-store-c --release` into the matching
`lib/<goos>_<goarch>/` directory (on Windows, ensure that directory is on `PATH`
when running tests).

### Building from this repository (contributors)

This `bindings/go` directory is the development source. To build it directly,
compile the C ABI and stage the library into the per-platform directory cgo
links against:

```bash
cargo build -p wickra-feature-store-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_feature_store.so bindings/go/lib/linux_amd64/
```

Then, with the library on the loader path, run `go test ./...` from this directory.

## Quick start

```go
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra-feature-store/bindings/go"
)

func main() {
	spec := `{"universe":["AAA"],` +
		`"features":[{"kind":"indicator","name":"Sma","params":[2]},{"kind":"price","field":"close"}],` +
		`"labels":[{"kind":"forward_return","horizon":1}]}`

	store, err := wickra.New(spec)
	if err != nil {
		panic(err)
	}
	defer store.Close()

	resp, err := store.Command(`{"cmd":"build_batch","data":{"AAA":[{"ts":0,"open":100,"high":100,"low":100,"close":100,"volume":1}]}}`)
	if err != nil {
		panic(err)
	}
	fmt.Println(resp)
}
```

### Surface

- **`New(specJSON string) (*FeatureStore, error)`** — build a feature store from
  a spec JSON. Returns an error if the spec is invalid. Call `Close` when done.
- **`(*FeatureStore) Command(cmdJSON string) (string, error)`** — apply a command
  envelope (`{"cmd":"...", ...}`) and return the response JSON. Commands:
  `set_spec`, `push`, `push_batch`, `build`, `build_batch`, `labels`, `reset`,
  `version`.
- **`Version() string`** — the crate version.

Domain errors (a bad command, an unknown command name) come back as an
`{"ok": false, "error": ...}` response, not as a returned `error`. The `error` is
reserved for hard failures at the C ABI boundary. Arrow / Parquet output is a
binary file format and is not available over this JSON surface; use the
`wickra-feature-store` CLI for columnar output.

### Determinism

The response bytes are identical across languages and between the parallel and
sequential build paths, because the whole feature fold lives once in the Rust
core and this binding forwards its JSON verbatim.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of cgo over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-feature-store/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-feature-store>
- **Docs** (guides, spec reference, cookbook): <https://feature-store.wickra.org>
- **Runnable example:** [`examples/go/`](https://github.com/wickra-lib/wickra-feature-store/tree/main/examples/go)

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
