# Wickra Feature Store examples — Go

Runnable Go examples for the [Wickra Feature Store Go binding](../../bindings/go). The binding links against the
prebuilt C ABI library, so build and stage it once before running anything:

```bash
cargo build -p wickra-feature-store-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_feature_store.so bindings/go/lib/linux_amd64/
```

## Run

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

## The examples

| Example | What it does |
|---------|--------------|
| `build_features.go` | A runnable Go example: build a feature matrix through the binding. |
