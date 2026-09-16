# Fuzzing Wickra Feature Store

[`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) harnesses for the parsing and stateful entry points of Wickra Feature Store. Fuzzing requires a nightly Rust toolchain; CI runs every target for 30 seconds on the family's pinned `nightly-2026-07-01`.

## Setup

```bash
cargo install cargo-fuzz
rustup toolchain install nightly-2026-07-01
```

The date is the family's fuzz nightly, pinned in `ci.yml`: a rolling `nightly`
regressed with a codegen ICE unrelated to this code, so every repository moves
the date together, on purpose.

## Targets

| Target | What it exercises |
| --- | --- |
| `spec_parse` | The spec-parsing surface: arbitrary bytes are parsed as a `FeatureSpec` (JSON). |
| `feature_eval` | The feature/label column-key surface: arbitrary bytes are parsed as a `Feature` or `Label` (JSON), and their canonical keys are computed. |
| `build_matrix` | The numeric fold: arbitrary bytes become a candle path that is folded through a fixed multi-feature, multi-label spec. |
| `label_compute` | The label formulas directly: arbitrary bytes become price arrays and an (index, horizon) pair fed to `forward_return` and `triple_barrier`. |

## Run

```bash
# From the repository root:
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu spec_parse
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu feature_eval
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu build_matrix
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu label_compute
```

Each run continues until a crash is found or it is interrupted. A short
time-boxed smoke run is what CI does:

```bash
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu spec_parse -- -max_total_time=30
```

The expectation for every target is that it never panics: malformed or
adversarial input must surface as an `Err` or an in-band error, never a crash.
