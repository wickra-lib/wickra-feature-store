// A runnable .NET example: build a feature matrix through the binding.
//
//   cargo build --release -p wickra-feature-store-c
//   dotnet run --project examples/csharp/BuildFeatures

using System.Text.Json;
using Wickra.FeatureStore;

const string spec =
    "{\"universe\":[\"AAA\",\"BBB\"],\"features\":[" +
    "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[2]}," +
    "{\"kind\":\"price\",\"field\":\"close\"}]," +
    "\"labels\":[{\"kind\":\"forward_return\",\"horizon\":1}]}";

static object Candle(int time, double close) =>
    new { time, open = close, high = close, low = close, close, volume = 1.0 };

static object[] Series(double[] closes)
{
    var bars = new object[closes.Length];
    for (int i = 0; i < closes.Length; i++)
    {
        bars[i] = Candle(i + 1, closes[i]);
    }
    return bars;
}

using var store = new FeatureStore(spec);

string cmd = JsonSerializer.Serialize(new
{
    cmd = "build_batch",
    data = new Dictionary<string, object[]>
    {
        ["AAA"] = Series([10, 11, 12]),
        ["BBB"] = Series([20, 22, 24]),
    },
});

string response = store.Command(cmd);
using JsonDocument matrix = JsonDocument.Parse(response);

Console.WriteLine($"wickra-feature-store {FeatureStore.Version()}");
Console.WriteLine($"rows: {matrix.RootElement.GetProperty("rows").GetInt32()}");
Console.WriteLine(response);
