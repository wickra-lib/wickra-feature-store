# Wickra Feature Store examples

A runnable "build a feature matrix" example in every language. Each one builds a
feature store from the same spec — a 2-period SMA, the close price, and a
1-bar forward-return label — over a two-symbol inline universe (`AAA` rising
`10 → 11 → 12`, `BBB` rising `20 → 22 → 24`) and prints the resulting matrix.

## What every example prints

Every example prints the version and the feature matrix. The matrix has three
columns — `Sma(2)`, `price.close`, `fwd_return(1)` — and six rows (three bars
per symbol, `AAA` before `BBB` in sorted order). The warmup bar has a `null`
SMA and the last bar of each symbol has a `null` label (no future bar), for
example:

```text
wickra-feature-store 0.1.1
columns: ["Sma(2)", "price.close", "fwd_return(1)"]
rows: 6
{"columns":["Sma(2)","price.close","fwd_return(1)"], ...}
```

## Rust — `examples/rust/`

As the CI examples job runs it, from the repository root:

```bash
cargo run -q -p wickra-feature-store-example
```

| Example | What it does |
| --- | --- |
| `src/main.rs` | A runnable Rust example: build a feature matrix from a small universe with the native `build` API and print the resulting matrix JSON. |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-feature-store-c --release`), then build and run
the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `build_features.c` | A minimal C example: build a feature matrix through the wickra-feature-store |
| `build_features.cpp` | A minimal C++ example: build a feature matrix over a two-symbol universe. |

## C# — `examples/csharp/`

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/BuildFeatures
```

| Example | What it does |
| --- | --- |
| `BuildFeatures/Program.cs` | A runnable .NET example: build a feature matrix through the binding. |

## Go — `examples/go/`

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

| Example | What it does |
| --- | --- |
| `build_features.go` | A runnable Go example: build a feature matrix through the binding. |

## R — `examples/r/`

As the CI examples job runs it, from the repository root:

```bash
R CMD INSTALL bindings/r
Rscript examples/r/build_features.R
```

| Example | What it does |
| --- | --- |
| `build_features.R` | A runnable R example: build a feature matrix through the binding. |

## Java — `examples/java/`

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q package -DskipTests
javac -cp bindings/java/target/classes examples/java/BuildFeatures.java -d examples/java/out
java --enable-native-access=ALL-UNNAMED  -Dnative.lib.dir="$PWD/target/release"  -cp "bindings/java/target/classes:examples/java/out" BuildFeatures
```

| Example | What it does |
| --- | --- |
| `BuildFeatures.java` | A runnable Java example: build a feature matrix through the binding. |

## Python — `examples/python/`

As the CI examples job runs it, from the repository root:

```bash
python -m pip install --require-hashes -r .github/requirements/ci-dev-py3.txt
( cd bindings/python && maturin build --release --out dist )
python -m pip install --no-index --find-links bindings/python/dist wickra-feature-store
python examples/python/build_features.py
```

| Example | What it does |
| --- | --- |
| `build_features.py` | A runnable Python example: build a feature matrix through the binding. |

## Node.js — `examples/node/`

As the CI examples job runs it, from the repository root:

```bash
( cd bindings/node && npm install --no-audit --no-fund && npx napi build --platform --release )
( cd examples/node && npm install --no-audit --no-fund )
node examples/node/build_features.js
```

| Example | What it does |
| --- | --- |
| `build_features.js` | A runnable Node.js example: build a feature matrix through the binding. |

## WASM — `examples/wasm/`

Build the WASM package, serve the repository root, and open the page in a browser;
the module script inside it is what runs (CI parses it with `node --check`):

```bash
wasm-pack build bindings/wasm --target web
python -m http.server 8000     # then open http://localhost:8000/examples/wasm/
```

| Example | What it does |
| --- | --- |
| `build.html` | A runnable example against this binding. |

## Example datasets

The examples read from [`examples/data/`](data/): . The
cross-language golden fixtures, which every binding is checked against byte for
byte, live in [`../golden/`](../golden).
