<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Feature Store — turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 streaming indicators, deterministic across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/ci.svg)](https://github.com/wickra-lib/wickra-feature-store/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-feature-store)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/release.svg)](https://github.com/wickra-lib/wickra-feature-store/releases/latest)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-feature-store/license.svg)](https://github.com/wickra-lib/wickra-feature-store#license)

# Wickra Feature Store — C / C++

---

**Turn OHLCV and microstructure event streams into ML-ready feature matrices over 497 O(1) streaming indicators — for C / C++. `cargo build -p wickra-feature-store-c --release` — a prebuilt shared/static library plus a generated `wickra_feature_store.h`, no system dependencies.**

The C ABI is the hub every C-capable language (C, C++, C#, Go, Java, R) links
against. It exposes `wickra-feature-store-core` as a tiny, JSON-shaped surface built as
both a `cdylib` (dynamic library) and a `staticlib`.

## Install

Grab the prebuilt header + library for your platform from the
[GitHub releases](https://github.com/wickra-lib/wickra-feature-store/releases) — each archive
has `wickra_feature_store.h`, the C++ wrapper where the binding ships one, and the shared/static
library — or build from source:

```bash
cargo build -p wickra-feature-store-c --release
# -> target/release/libwickra_feature_store.{so,dylib} or wickra_feature_store.dll (+ import lib) + a staticlib
```

Then compile against the header and link the library.

## Quick start

[`examples/c/build_features.c`](https://github.com/wickra-lib/wickra-feature-store/blob/main/examples/c/build_features.c) is the runnable example the CI smoke job executes; in full:

```c
/* A minimal C example: build a feature matrix through the wickra-feature-store
 * C ABI. */
#include <stdio.h>
#include <stdlib.h>

#include "wickra_feature_store.h"

static const char *SPEC =
    "{\"universe\":[\"AAA\",\"BBB\"],\"features\":["
    "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[2]},"
    "{\"kind\":\"price\",\"field\":\"close\"}],"
    "\"labels\":[{\"kind\":\"forward_return\",\"horizon\":1}]}";

static const char *CMD =
    "{\"cmd\":\"build_batch\",\"data\":{"
    "\"AAA\":["
    "{\"ts\":1,\"open\":10,\"high\":10,\"low\":10,\"close\":10,\"volume\":1},"
    "{\"ts\":2,\"open\":11,\"high\":11,\"low\":11,\"close\":11,\"volume\":1},"
    "{\"ts\":3,\"open\":12,\"high\":12,\"low\":12,\"close\":12,\"volume\":1}],"
    "\"BBB\":["
    "{\"ts\":1,\"open\":20,\"high\":20,\"low\":20,\"close\":20,\"volume\":1},"
    "{\"ts\":2,\"open\":22,\"high\":22,\"low\":22,\"close\":22,\"volume\":1},"
    "{\"ts\":3,\"open\":24,\"high\":24,\"low\":24,\"close\":24,\"volume\":1}]}}";

int main(void) {
    WickraFeatureStore *store = wickra_feature_store_new(SPEC);
    if (!store) {
        fprintf(stderr, "failed to build feature store\n");
        return 1;
    }

    /* Length-out protocol: learn the length, then read into a caller buffer. */
    int len = wickra_feature_store_command(store, CMD, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        wickra_feature_store_free(store);
        return 1;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        wickra_feature_store_free(store);
        return 1;
    }
    wickra_feature_store_command(store, CMD, buf, (size_t)len + 1);

    printf("wickra-feature-store %s\n", wickra_feature_store_version());
    printf("matrix: %s\n", buf);

    free(buf);
    wickra_feature_store_free(store);
    return 0;
}
```

### Surface

```c
#include "wickra_feature_store.h"

WickraFeatureStore *wickra_feature_store_new(const char *spec_json);
void                wickra_feature_store_free(WickraFeatureStore *handle);
int32_t             wickra_feature_store_command(WickraFeatureStore *handle,
                                                 const char *cmd_json,
                                                 char *out, size_t cap);
const char         *wickra_feature_store_version(void);
```

- **`wickra_feature_store_new`** builds a feature store from a spec JSON. Returns
  `NULL` if the argument is null, not UTF-8, or not a valid spec.
- **`wickra_feature_store_free`** destroys a handle (null is a no-op).
- **`wickra_feature_store_command`** applies a command JSON and writes the
  response JSON into the caller's buffer using a length-out protocol (below).
- **`wickra_feature_store_version`** returns a static, NUL-terminated version
  string (do not free).

### Command / response protocol

Everything after construction goes through `wickra_feature_store_command`.
Commands are JSON objects with a `"cmd"` field: `set_spec`, `push`, `push_batch`,
`build`, `build_batch`, `labels`, `reset`, `version`. Responses are JSON, e.g. a
feature matrix `{"columns":[...],"index":[...],"data":[...],"rows":N}` for a
build, or `{"ok":true}` for a mutation.

The response is returned via a caller-owned buffer with a length-out protocol —
the callee never allocates memory the caller must free:

1. Call with `out = NULL`, `cap = 0` to learn the response length `len`
   (excluding the terminating NUL).
2. Allocate `len + 1` bytes and call again; the response plus a NUL is written.

Whenever `len < cap`, the response is written on that call, so a
sufficiently-large buffer needs only one call.

Return codes:

| Return   | Meaning                                             |
|----------|-----------------------------------------------------|
| `>= 0`   | Response length in bytes (excluding the NUL).       |
| `-1`     | A required pointer (`handle` or `cmd_json`) is null. |
| `-2`     | `cmd_json` is not valid UTF-8.                       |
| `-3`     | A panic was caught at the boundary.                 |

Domain errors (a bad spec, an unknown command) are **not** negative — they come
back in-band as `{"ok":false,"error":...}` JSON in the buffer.

Arrow / Parquet output is a binary file format and is intentionally **not**
available over this boundary; use the [`wickra-feature-store`
CLI](https://github.com/wickra-lib/wickra-feature-store) or the native Rust
`arrow` feature for columnar output.

### Header generation

`include/wickra_feature_store.h` is generated with [cbindgen] and committed; CI
fails if it drifts from the source. Regenerate after changing the ABI:

```sh
cbindgen --config cbindgen.toml --crate wickra-feature-store-c --output include/wickra_feature_store.h
```

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the C ABI itself, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-feature-store/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-feature-store>
- **Docs** (guides, spec reference, cookbook): <https://feature-store.wickra.org>
- **Runnable example:** [`examples/c/`](https://github.com/wickra-lib/wickra-feature-store/tree/main/examples/c)

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

[cbindgen]: https://github.com/mozilla/cbindgen
