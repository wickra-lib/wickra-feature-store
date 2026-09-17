<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/npm.svg)](https://www.npmjs.com/package/wickra-feature-store-wasm)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](https://github.com/wickra-lib/wickra-feature-store#license)

# Wickra Feature Store — WASM

---

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — for WASM. `npm install wickra-feature-store-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

WebAssembly bindings for the Wickra feature-matrix core, compiled from Rust with
[wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/). A `FeatureStore` is
built from a spec JSON and driven by command JSONs over a JSON boundary, so a
browser front-end runs against the exact same core as every other Wickra Feature
Store binding.

## Install

```bash
npm install wickra-feature-store-wasm
```

### Building from this repository (contributors)

```bash
wasm-pack build --target web      # for a browser bundler
wasm-pack build --target nodejs   # for node:test / Node.js
```

The output lands in `pkg/`.

## Quick start

```js
import init, { FeatureStore } from "wickra-feature-store-wasm";

await init();

const spec = JSON.stringify({
  universe: ["AAA"],
  features: [
    { kind: "indicator", name: "Sma", params: [2] },
    { kind: "price", field: "close" },
  ],
  labels: [{ kind: "forward_return", horizon: 1 }],
});

const store = new FeatureStore(spec);
const candle = (ts, close) => ({ ts, open: close, high: close, low: close, close, volume: 1.0 });

const response = store.command(JSON.stringify({
  cmd: "build_batch",
  data: { AAA: [candle(0, 100.0), candle(1, 110.0), candle(2, 121.0)] },
}));

const matrix = JSON.parse(response);
console.log(matrix.columns); // ['Sma(2)', 'price.close', 'fwd_return(1)']
```

### Surface

- **`new FeatureStore(specJson)`** — build a feature store from a spec JSON
  (throws if the spec is invalid).
- **`store.command(cmdJson) -> string`** — apply a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `push`, `push_batch`, `build`, `build_batch`, `labels`, `reset`, `version`.
- **`store.version() -> string`** and the module-level **`version()`** — the
  crate version.

An invalid spec, a malformed command, or an unknown command name throws; a
successful command returns the response JSON.

Arrow / Parquet output is a native-only binary format and is **not** available in
the browser; use the `wickra-feature-store` CLI for columnar output.

### Determinism

The feature fold runs sequentially in the browser sandbox (no rayon thread pool),
which is byte-identical to the native run — the exact cross-language golden
invariant. The response bytes match every other binding.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of wasm-bindgen, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-feature-store/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-feature-store>
- **Docs** (guides, spec reference, cookbook): <https://feature-store.wickra.org>
- **Runnable example:** [`examples/wasm/`](https://github.com/wickra-lib/wickra-feature-store/tree/main/examples/wasm)

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
