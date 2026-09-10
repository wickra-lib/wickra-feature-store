# wickra-feature-store (C#)

.NET bindings for [`wickra-feature-store`](https://github.com/wickra-lib/wickra-feature-store)
over the C ABI hub, via source-generated P/Invoke. Build a `FeatureStore` from a
spec JSON, drive it with command JSON and read back the feature matrix — the same
protocol the CLI and every other binding speak, returning the same bytes.

```csharp
using Wickra.FeatureStore;

const string spec = """
{"universe":["AAA"],"timeframe":"1h",
 "features":[{"kind":"indicator","name":"Rsi","params":[14]},
             {"kind":"price","field":"close"}],
 "labels":[{"kind":"forward_return","horizon":5}]}
""";

using var store = new FeatureStore(spec);
store.Command("""{"cmd":"push","symbol":"AAA","candle":{"ts":1700000000,
  "open":100,"high":101,"low":99,"close":100.5,"volume":1000}}""");
string matrix = store.Command("""{"cmd":"build"}""");
```

A column whose indicator reads a side feed — a reference series, a derivatives
tick, an order book, the bar's trades, the market cross-section — needs that feed
supplied, either on the `push` (`"feeds":{…}`) or per symbol in a `build_batch`
payload. A spec naming one without it is refused by name rather than answered
with a column that is `null` for its whole length.

Requires .NET 8+. The native library (`wickra_feature_store`) must be resolvable
on the loader path — `PATH` on Windows, `LD_LIBRARY_PATH` on Linux,
`DYLD_LIBRARY_PATH` on macOS — or beside the assembly, where the bundled
resolver finds it.

Licensed under either of [MIT](https://github.com/wickra-lib/wickra-feature-store/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/wickra-lib/wickra-feature-store/blob/main/LICENSE-APACHE) at your option.
