# Wickra Feature Store — Go

Go bindings for the Wickra feature-matrix core over its C ABI hub via cgo. A
`FeatureStore` is built from a spec JSON and driven over a JSON boundary, so the
result is byte-identical to every other Wickra Feature Store binding.

## Install

```bash
go get github.com/wickra-lib/wickra-feature-store/bindings/go
```

The prebuilt C ABI library is staged per platform under `lib/<goos>_<goarch>/`
and the header is vendored under `include/`. For a local build, copy the library
built by `cargo build -p wickra-feature-store-c --release` into the matching
`lib/<goos>_<goarch>/` directory (on Windows, ensure that directory is on `PATH`
when running tests).

## Usage

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

## Surface

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

## Determinism

The response bytes are identical across languages and between the parallel and
sequential build paths, because the whole feature fold lives once in the Rust
core and this binding forwards its JSON verbatim.

## See also

- The main project: <https://github.com/wickra-lib/wickra-feature-store>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
