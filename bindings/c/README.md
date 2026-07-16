# Wickra Feature Store — C ABI

The C ABI is the hub every C-capable language (C, C++, C#, Go, Java, R) links
against. It exposes `feature-store-core` as a tiny, JSON-shaped surface built as
both a `cdylib` (dynamic library) and a `staticlib`.

## Surface

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

## Command / response protocol

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

## Header generation

`include/wickra_feature_store.h` is generated with [cbindgen] and committed; CI
fails if it drifts from the source. Regenerate after changing the ABI:

```sh
cbindgen --config cbindgen.toml --crate wickra-feature-store-c --output include/wickra_feature_store.h
```

[cbindgen]: https://github.com/mozilla/cbindgen
