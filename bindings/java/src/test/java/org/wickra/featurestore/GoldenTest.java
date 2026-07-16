package org.wickra.featurestore;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

// The cross-language golden invariant seen from Java: the same command yields
// byte-identical output across calls, and streaming a spec bar-by-bar matches the
// batch build. The response bytes are what every other binding produces too,
// because the whole feature fold lives once in the Rust core and this binding
// forwards its JSON verbatim.
class GoldenTest {
    @Test
    void buildBatchIsByteIdenticalAcrossCalls() {
        try (FeatureStore a = new FeatureStore(FeatureStoreTest.SPEC);
                FeatureStore b = new FeatureStore(FeatureStoreTest.SPEC)) {
            assertEquals(a.command(FeatureStoreTest.buildBatchCmd()), b.command(FeatureStoreTest.buildBatchCmd()));
        }
    }

    @Test
    void streamingBuildMatchesBatch() {
        String batch;
        try (FeatureStore batchStore = new FeatureStore(FeatureStoreTest.SPEC)) {
            batch = batchStore.command(FeatureStoreTest.buildBatchCmd());
        }

        try (FeatureStore streamed = new FeatureStore(FeatureStoreTest.SPEC)) {
            for (int ts = 0; ts <= 2; ts++) {
                double close = new double[] {100.0, 110.0, 121.0}[ts];
                String push = "{\"cmd\":\"push\",\"symbol\":\"AAA\",\"candle\":"
                        + FeatureStoreTest.candle(ts, close) + "}";
                streamed.command(push);
            }
            String built = streamed.command("{\"cmd\":\"build\"}");
            assertEquals(batch, built);
        }
    }
}
