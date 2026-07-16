using System.Text.Json;
using Wickra.FeatureStore;
using Xunit;

namespace WickraFeatureStore.Tests;

public class FeatureStoreTests
{
    internal const string Spec =
        "{\"universe\":[\"AAA\"]," +
        "\"features\":[{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[2]},{\"kind\":\"price\",\"field\":\"close\"}]," +
        "\"labels\":[{\"kind\":\"forward_return\",\"horizon\":1}]}";

    private static object Candle(int ts, double close) => new
    {
        ts,
        open = close,
        high = close,
        low = close,
        close,
        volume = 1.0,
    };

    internal static object[] Candles() => [Candle(0, 100.0), Candle(1, 110.0), Candle(2, 121.0)];

    internal static string BuildBatchCmd() => JsonSerializer.Serialize(new
    {
        cmd = "build_batch",
        data = new Dictionary<string, object[]> { ["AAA"] = Candles() },
    });

    [Fact]
    public void Version_IsNonEmpty()
    {
        Assert.False(string.IsNullOrEmpty(FeatureStore.Version()));
    }

    [Fact]
    public void BuildBatch_ReturnsMatrix()
    {
        using var store = new FeatureStore(Spec);
        JsonElement matrix = JsonDocument.Parse(store.Command(BuildBatchCmd())).RootElement;

        string[] columns = [.. matrix.GetProperty("columns").EnumerateArray().Select(c => c.GetString()!)];
        Assert.Equal(["Sma(2)", "price.close", "fwd_return(1)"], columns);
        Assert.Equal(matrix.GetProperty("rows").GetInt32(), matrix.GetProperty("data").GetArrayLength());
    }

    [Fact]
    public void InvalidSpec_Throws()
    {
        Assert.Throws<ArgumentException>(() => new FeatureStore("{ not valid json"));
    }

    [Fact]
    public void UnknownCommand_IsInBandError()
    {
        using var store = new FeatureStore(Spec);
        // The C ABI hub folds a domain error into {"ok":false,...} JSON, so an
        // unknown command surfaces in-band rather than as an exception.
        string raw = store.Command("{\"cmd\":\"nope\"}");
        Assert.Contains("\"ok\":false", raw);
    }
}
