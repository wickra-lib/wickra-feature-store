using System.Text.Json;
using Wickra.FeatureStore;
using Xunit;

namespace WickraFeatureStore.Tests;

// The cross-language golden invariant seen from C#: the same command yields
// byte-identical output across calls, and streaming a spec bar-by-bar matches the
// batch build. The response bytes are what every other binding produces too,
// because the whole feature fold lives once in the Rust core and this binding
// forwards its JSON verbatim.
public class GoldenTests
{
    [Fact]
    public void BuildBatch_IsByteIdenticalAcrossCalls()
    {
        using var a = new FeatureStore(FeatureStoreTests.Spec);
        using var b = new FeatureStore(FeatureStoreTests.Spec);
        Assert.Equal(a.Command(FeatureStoreTests.BuildBatchCmd()), b.Command(FeatureStoreTests.BuildBatchCmd()));
    }

    [Fact]
    public void StreamingBuild_MatchesBatch()
    {
        using var batchStore = new FeatureStore(FeatureStoreTests.Spec);
        string batch = batchStore.Command(FeatureStoreTests.BuildBatchCmd());

        using var streamed = new FeatureStore(FeatureStoreTests.Spec);
        foreach (JsonElement candle in JsonSerializer.SerializeToDocument(FeatureStoreTests.Candles()).RootElement.EnumerateArray())
        {
            string push = JsonSerializer.Serialize(new
            {
                cmd = "push",
                symbol = "AAA",
                candle = JsonSerializer.Deserialize<JsonElement>(candle.GetRawText()),
            });
            streamed.Command(push);
        }
        string built = streamed.Command("{\"cmd\":\"build\"}");

        Assert.Equal(batch, built);
    }
}
