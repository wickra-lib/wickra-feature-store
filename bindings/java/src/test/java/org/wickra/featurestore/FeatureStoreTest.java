package org.wickra.featurestore;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class FeatureStoreTest {
    static final String SPEC =
            "{\"universe\":[\"AAA\"],"
                    + "\"features\":[{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[2]},"
                    + "{\"kind\":\"price\",\"field\":\"close\"}],"
                    + "\"labels\":[{\"kind\":\"forward_return\",\"horizon\":1}]}";

    static String candle(int ts, double close) {
        return "{\"ts\":" + ts
                + ",\"open\":" + close
                + ",\"high\":" + close
                + ",\"low\":" + close
                + ",\"close\":" + close
                + ",\"volume\":1.0}";
    }

    static String candles() {
        return "[" + candle(0, 100.0) + "," + candle(1, 110.0) + "," + candle(2, 121.0) + "]";
    }

    static String buildBatchCmd() {
        return "{\"cmd\":\"build_batch\",\"data\":{\"AAA\":" + candles() + "}}";
    }

    @Test
    void versionIsNonEmpty() {
        assertFalse(FeatureStore.version().isEmpty());
    }

    @Test
    void buildBatchReturnsMatrix() {
        try (FeatureStore store = new FeatureStore(SPEC)) {
            String matrix = store.command(buildBatchCmd());
            assertTrue(matrix.contains("\"Sma(2)\""), matrix);
            assertTrue(matrix.contains("\"price.close\""), matrix);
            assertTrue(matrix.contains("\"fwd_return(1)\""), matrix);
            assertTrue(matrix.contains("\"rows\":3"), matrix);
        }
    }

    @Test
    void invalidSpecThrows() {
        assertThrows(IllegalArgumentException.class, () -> new FeatureStore("{ not valid json"));
    }

    @Test
    void unknownCommandIsInBandError() {
        try (FeatureStore store = new FeatureStore(SPEC)) {
            // The C ABI hub folds a domain error into {"ok":false,...} JSON, so an
            // unknown command surfaces in-band rather than as an exception.
            String raw = store.command("{\"cmd\":\"nope\"}");
            assertTrue(raw.contains("\"ok\":false"), raw);
        }
    }
}
