<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/nuget.svg)](https://www.nuget.org/packages/Wickra.FeatureStore)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](https://github.com/wickra-lib/wickra-feature-store#license)

# Wickra Feature Store — C#

---

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — for C#. `dotnet add package Wickra.FeatureStore` — prebuilt native library, no system dependencies.**

.NET bindings for [`wickra-feature-store`](https://github.com/wickra-lib/wickra-feature-store)
over the C ABI hub, via source-generated P/Invoke. Build a `FeatureStore` from a
spec JSON, drive it with command JSON and read back the feature matrix — the same
protocol the CLI and every other binding speak, returning the same bytes.

## Install

```bash
dotnet add package Wickra.FeatureStore
```

The native library ships prebuilt per platform under `runtimes/<rid>/native/`,
selected automatically. There is nothing to compile. Targets .NET 8 and later.

Requires .NET 8+. The native library (`wickra_feature_store`) must be resolvable
on the loader path — `PATH` on Windows, `LD_LIBRARY_PATH` on Linux,
`DYLD_LIBRARY_PATH` on macOS — or beside the assembly, where the bundled
resolver finds it.

## Quick start

```csharp
using Wickra.FeatureStore;

const string spec = """
{"universe":["AAA"],"timeframe":"1h",
 "features":[{"kind":"indicator","name":"Rsi","params":[14]},
             {"kind":"price","field":"close"}],
 "labels":[{"kind":"forward_return","horizon":5}]}
""";

using var store = new FeatureStore(spec);
store.Command("""{"cmd":"push","symbol":"AAA","candle":{"ts":1700000000,
  "open":100,"high":101,"low":99,"close":100.5,"volume":1000}}""");
string matrix = store.Command("""{"cmd":"build"}""");
```

A column whose indicator reads a side feed — a reference series, a derivatives
tick, an order book, the bar's trades, the market cross-section — needs that feed
supplied, either on the `push` (`"feeds":{…}`) or per symbol in a `build_batch`
payload. A spec naming one without it is refused by name rather than answered
with a column that is `null` for its whole length.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of `[LibraryImport]` P/Invoke over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-feature-store/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-feature-store>
- **Docs** (guides, spec reference, cookbook): <https://feature-store.wickra.org>
- **Runnable example:** [`examples/csharp/`](https://github.com/wickra-lib/wickra-feature-store/tree/main/examples/csharp)

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
