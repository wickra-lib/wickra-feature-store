# Wickra Feature Store examples — C#

Runnable C# examples for the [Wickra Feature Store C# binding](../../bindings/csharp). The binding consumes the C ABI
library through P/Invoke, so build it once before running anything:

```bash
cargo build -p wickra-feature-store-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/BuildFeatures
```

## The examples

| Example | What it does |
|---------|--------------|
| `BuildFeatures/Program.cs` | A runnable .NET example: build a feature matrix through the binding. |
