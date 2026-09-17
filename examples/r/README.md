# Wickra Feature Store examples — R

Runnable R examples for the [Wickra Feature Store R binding](../../bindings/r). The package compiles a thin
`.Call` glue layer against the C ABI library, so build the library and install
the package first (the CI examples job does exactly this):

```bash
cargo build -p wickra-feature-store-c --release
R CMD INSTALL bindings/r
```

## Run

```bash
Rscript examples/r/build_features.R
```

## The examples

| Example | What it does |
|---------|--------------|
| `build_features.R` | A runnable R example: build a feature matrix through the binding. |
