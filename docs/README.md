# Documentation

These pages are the guides that live beside the code, because they describe how
this repository behaves and have to change in the same commit the behaviour does.

| Page | What it answers |
|------|-----------------|
| [FEATURES.md](FEATURES.md) | The shape of a `FeatureSpec`: feature columns, price fields, microstructure metrics, column keys |
| [FEEDS.md](FEEDS.md) | The side feeds: which indicator needs what, the batch and per-bar shapes, and why a spec is refused rather than answered with a dead column |
| [LABELS.md](LABELS.md) | Forward return and the triple barrier: horizons, the bars they consume, and the cells they leave `null` |
| [SCALING.md](SCALING.md) | Z-score and min-max, applied per feature column after the fold |
| [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) | JSON, CSV, Arrow and Parquet, and which of them cross the C ABI |
| [STREAMING.md](STREAMING.md) | Pushing bar by bar against building in batch, and where the two are identical |
| [Cookbook.md](Cookbook.md) | Worked feature matrices |

The API reference for each language is generated from the source rather than
committed here — `cargo doc` for Rust, the `.d.ts` beside the Node binding, the
docstrings in the Python module, the C header. Keeping a second copy in this
repository would drift from the code that generates it, and a reader opening
`docs/` would have no way to tell which copy was current.

The indicator library the feature store resolves names through documents itself
at <https://docs.wickra.org>.

What stays here is what a generator cannot produce: the meaning of a field, the
reason a case is refused rather than answered, and the worked examples.

Elsewhere in the repository:

- [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — the crate and binding layout
- [`../BENCHMARKS.md`](../BENCHMARKS.md) — what is measured and how
- [`../golden/README.md`](../golden/README.md) — the cross-language corpus and how to regenerate it
- [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — how to build, test and propose a change
- [`../THREAT_MODEL.md`](../THREAT_MODEL.md) — what the feature store does and does not touch
