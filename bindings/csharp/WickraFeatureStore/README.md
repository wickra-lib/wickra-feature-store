# Wickra Feature Store — C#

.NET bindings for the Wickra feature-matrix core over its C ABI hub. A
`FeatureStore` is built from a spec JSON and driven over a JSON boundary, so the
result is byte-identical to every other Wickra Feature Store binding.

## Install

```bash
dotnet add package Wickra.FeatureStore
```

The package ships the native C ABI library per runtime identifier under
`runtimes/<rid>/native/`. For a local build, `cargo build -p wickra-feature-store-c --release`
places the library in `target/release/`; the bundled `DllImportResolver` probes
the Cargo `target/` tree, so tests and apps in the repo find it without extra
steps.

## Usage

```csharp
using Wickra.FeatureStore;

const string spec = """
{"universe":["AAA"],
 "features":[{"kind":"indicator","name":"Sma","params":[2]},{"kind":"price","field":"close"}],
 "labels":[{"kind":"forward_return","horizon":1}]}
""";

using var store = new FeatureStore(spec);

string response = store.Command("""
{"cmd":"build_batch","data":{"AAA":[{"ts":0,"open":100,"high":100,"low":100,"close":100,"volume":1}]}}
""");
Console.WriteLine(response);
```

## Surface

- **`new FeatureStore(string specJson)`** — build a feature store from a spec
  JSON. Throws `ArgumentException` if the spec is invalid. Implements
  `IDisposable`; dispose it when done.
- **`string Command(string cmdJson)`** — apply a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `push`, `push_batch`, `build`, `build_batch`, `labels`, `reset`, `version`.
- **`static string Version()`** — the crate version.

Domain errors (a bad command, an unknown command name) come back as an
`{"ok": false, "error": ...}` response, not as an exception. Exceptions are
reserved for an invalid spec at construction and hard failures at the C ABI
boundary. Arrow / Parquet output is a binary file format and is not available
over this JSON surface; use the `wickra-feature-store` CLI for columnar output.

## Determinism

The response bytes are identical across languages and between the parallel and
sequential build paths, because the whole feature fold lives once in the Rust
core and this binding forwards its JSON verbatim.

## See also

- The main project: <https://github.com/wickra-lib/wickra-feature-store>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either [MIT](../../../LICENSE-MIT) or
[Apache-2.0](../../../LICENSE-APACHE), at your option.
