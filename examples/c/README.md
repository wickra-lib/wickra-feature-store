# Wickra Feature Store — C / C++ examples

The Wickra Feature Store C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra_feature_store.h`](../../bindings/c/include/wickra_feature_store.h)). Any C-capable
language links against the same artifact; these examples show the plain-C path
and, through [`wickra_feature_store.hpp`](../../bindings/c/include/wickra_feature_store.hpp), the C++ one.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-feature-store-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_feature_store.so`     | `-lwickra_feature_store` |
| macOS    | `libwickra_feature_store.dylib`  | `-lwickra_feature_store` |
| Windows (MSVC) | `wickra_feature_store.dll` | `wickra_feature_store.dll.lib` (import lib) |

A static library (`libwickra_feature_store.a` / `wickra_feature_store.lib`) is emitted alongside.

## Build and run the examples

### With CMake (portable, used by CI)

```sh
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/build_features.c -I bindings/c/include -L target/release -lwickra_feature_store -lm -o build_features
LD_LIBRARY_PATH=target/release ./build_features        # macOS: DYLD_LIBRARY_PATH

# Windows (MinGW gcc, linking the DLL directly)
gcc examples/c/build_features.c -I bindings/c/include target/release/wickra_feature_store.dll -lm -o build_features.exe
```

## The examples

| Example | What it does |
|---------|--------------|
| `build_features.c` | A minimal C example: build a feature matrix through the wickra-feature-store |
| `build_features.cpp` | A minimal C++ example: build a feature matrix over a two-symbol universe. |

## Usage shape

Every call follows the same handle discipline: construct from a spec JSON, drive
with command JSON, read the response, free the handle exactly once. `wickra_feature_store.h` is
the whole contract; the C++ header, where one ships, wraps the handle in a
move-only RAII type. See [`bindings/c/README.md`](../../bindings/c/README.md).
