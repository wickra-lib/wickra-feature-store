# wickra-feature-store WASM examples

Browser demos for the `wickra-feature-store-wasm` binding.

The WASM build carries the whole feature core with `--no-default-features`: no
rayon, so the build is sequential rather than parallel — and byte-for-byte
identical to the parallel one, which is what the golden fixtures pin down. A
spec is data, not code, so the spec bytes on this page are the same ones
`examples/node/build_features.js` sends.

## Build

The module ships as a `wasm-pack` `--target web` bundle. Build it once from the
repository root:

```bash
wasm-pack build bindings/wasm --target web --release
```

That writes `bindings/wasm/pkg/` with the `.wasm` binary, the JS loader and the
TypeScript types. The demo imports the loader via
`../../bindings/wasm/pkg/wickra_feature_store_wasm.js`.

## Serve

ES-module imports need a real HTTP origin, not `file://`. Any static server from
the repository root works:

```bash
python -m http.server 8000
```

Then open `http://localhost:8000/examples/wasm/build.html`.

## Demos

| File | What it does |
| --- | --- |
| `build.html` | Builds a store from the shared spec (`Sma(2)`, the close, a one-bar forward return), folds the two-symbol inline universe (`AAA` 10/11/12, `BBB` 20/22/24) and renders the matrix, plus the raw JSON. The page counterpart of `examples/node/build_features.js`. |

## See also

- [examples/README.md](../README.md) — the same build in every other language.
- [bindings/wasm/README.md](../../bindings/wasm/README.md) — the module's API and
  what the sequential build does and does not carry.
