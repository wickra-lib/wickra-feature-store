# Wickra Feature Store — Java

JVM bindings for the Wickra feature-matrix core over its C ABI hub, using the
Foreign Function & Memory API (FFM / Panama). A `FeatureStore` is built from a
spec JSON and driven over a JSON boundary, so the result is byte-identical to
every other Wickra Feature Store binding.

## Requirements

- JDK 22+ (the FFM API is stable since Java 22). Run with
  `--enable-native-access=ALL-UNNAMED`.
- The native C ABI library, built by `cargo build -p wickra-feature-store-c`.
  The binding loads it from the directory named by the `native.lib.dir` system
  property (the Maven build points it at the workspace `target/debug`).

## Usage

```java
import org.wickra.featurestore.FeatureStore;

String spec = "{\"universe\":[\"AAA\"],"
    + "\"features\":[{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[2]},{\"kind\":\"price\",\"field\":\"close\"}],"
    + "\"labels\":[{\"kind\":\"forward_return\",\"horizon\":1}]}";

try (FeatureStore store = new FeatureStore(spec)) {
    String response = store.command(
        "{\"cmd\":\"build_batch\",\"data\":{\"AAA\":[{\"ts\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1}]}}");
    System.out.println(response);
}
```

## Surface

- **`new FeatureStore(String specJson)`** — build a feature store from a spec
  JSON. Throws `IllegalArgumentException` if the spec is invalid. Implements
  `AutoCloseable`; use try-with-resources.
- **`String command(String cmdJson)`** — apply a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `push`, `push_batch`, `build`, `build_batch`, `labels`, `reset`, `version`.
- **`static String version()`** — the crate version.

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

Dual-licensed under either [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
