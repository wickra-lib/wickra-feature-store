<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-feature-store)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](https://github.com/wickra-lib/wickra-feature-store#license)

# Wickra Feature Store — Java

---

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — for Java. `org.wickra:wickra-feature-store` — prebuilt native library inside the jar, no JNI, no system dependencies.**

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

## Install

Maven:

```xml
<dependency>
  <groupId>org.wickra</groupId>
  <artifactId>wickra-feature-store</artifactId>
  <version>0.1.1</version>
</dependency>
```

Gradle:

```kotlin
implementation("org.wickra:wickra-feature-store:0.1.1")
```

The native library ships prebuilt per platform inside the jar and is
extracted automatically on first use. There is nothing to compile.

## Quick start

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

### Surface

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

### Determinism

The response bytes are identical across languages and between the parallel and
sequential build paths, because the whole feature fold lives once in the Rust
core and this binding forwards its JSON verbatim.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the Java Foreign Function & Memory API over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-feature-store/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-feature-store>
- **Docs** (guides, spec reference, cookbook): <https://feature-store.wickra.org>
- **Runnable example:** [`examples/java/`](https://github.com/wickra-lib/wickra-feature-store/tree/main/examples/java)

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
