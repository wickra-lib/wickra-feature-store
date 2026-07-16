# Wickra Feature Store — WASM

WebAssembly bindings for the Wickra feature-matrix core, compiled from Rust with
[wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/). A `FeatureStore` is
built from a spec JSON and driven by command JSONs over a JSON boundary, so a
browser front-end runs against the exact same core as every other Wickra Feature
Store binding.

## Build

```bash
wasm-pack build --target web      # for a browser bundler
wasm-pack build --target nodejs   # for node:test / Node.js
```

The output lands in `pkg/`.

## Usage

```js
import init, { FeatureStore } from "./pkg/wickra_feature_store_wasm.js";

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

## Surface

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

## Determinism

The feature fold runs sequentially in the browser sandbox (no rayon thread pool),
which is byte-identical to the native run — the exact cross-language golden
invariant. The response bytes match every other binding.

## See also

- The main project: <https://github.com/wickra-lib/wickra-feature-store>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
