# Examples

A runnable "build a feature matrix" example in every language. Each one builds a
feature store from the same spec — a 2-period SMA, the close price, and a
1-bar forward-return label — over a two-symbol inline universe (`AAA` rising
`10 → 11 → 12`, `BBB` rising `20 → 22 → 24`) and prints the resulting matrix.

The examples are self-contained: the spec and candles are inline, so no external
files are loaded. The same spec and universe are mirrored as loadable fixtures
under [`data/`](data/) ([`data/specs/momentum.json`](data/specs/momentum.json)
and [`data/universe/`](data/universe/)) for adapting the examples to file input,
and the blessed cross-language golden corpus lives in [`../golden/`](../golden).

| Language | Path | Run |
|----------|------|-----|
| Rust | [`rust/`](rust/) | `cargo run -p wickra-feature-store-example` |
| Python | [`python/build_features.py`](python/build_features.py) | `pip install wickra-feature-store && python examples/python/build_features.py` |
| Node.js | [`node/`](node/) | `cd examples/node && npm install && node build_features.js` |
| C / C++ | [`c/`](c/) | see below |
| Go | [`go/`](go/) | `cd examples/go && go run .` |
| C# | [`csharp/BuildFeatures/`](csharp/BuildFeatures/) | `dotnet run --project examples/csharp/BuildFeatures` |
| Java | [`java/BuildFeatures.java`](java/BuildFeatures.java) | see the header comment |
| R | [`r/build_features.R`](r/build_features.R) | `Rscript examples/r/build_features.R` |

The native bindings (Python, Node.js) load their own compiled library. The
bindings that go through the C ABI (Go, C#, Java, R, and the C/C++ example
itself) need the C ABI library built first:

```bash
cargo build --release -p wickra-feature-store-c
```

## C / C++

The C and C++ examples build with CMake and run under ctest:

```bash
cargo build --release -p wickra-feature-store-c
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

On Windows the build copies `wickra_feature_store.dll` next to each executable,
since there is no rpath.

## Expected output

Every example prints the version and the feature matrix. The matrix has three
columns — `Sma(2)`, `price.close`, `fwd_return(1)` — and six rows (three bars
per symbol, `AAA` before `BBB` in sorted order). The warmup bar has a `null`
SMA and the last bar of each symbol has a `null` label (no future bar), for
example:

```text
wickra-feature-store 0.1.0
columns: ["Sma(2)", "price.close", "fwd_return(1)"]
rows: 6
{"columns":["Sma(2)","price.close","fwd_return(1)"], ...}
```
