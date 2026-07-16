// A runnable Java example: build a feature matrix through the binding.
//
//   cargo build -p wickra-feature-store-c
//   mvn -f bindings/java/pom.xml -q package -DskipTests
//   javac -cp bindings/java/target/classes examples/java/BuildFeatures.java -d examples/java/out
//   java --enable-native-access=ALL-UNNAMED \
//        -Dnative.lib.dir=target/debug \
//        -cp "bindings/java/target/classes;examples/java/out" BuildFeatures
import org.wickra.featurestore.FeatureStore;

public final class BuildFeatures {
    private static final String SPEC =
            "{\"universe\":[\"AAA\",\"BBB\"],\"features\":["
                    + "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[2]},"
                    + "{\"kind\":\"price\",\"field\":\"close\"}],"
                    + "\"labels\":[{\"kind\":\"forward_return\",\"horizon\":1}]}";

    private static String candle(int time, int close) {
        return "{\"time\":" + time + ",\"open\":" + close + ",\"high\":" + close
                + ",\"low\":" + close + ",\"close\":" + close + ",\"volume\":1}";
    }

    public static void main(String[] args) {
        try (FeatureStore store = new FeatureStore(SPEC)) {
            String cmd = "{\"cmd\":\"build_batch\",\"data\":{"
                    + "\"AAA\":[" + candle(1, 10) + "," + candle(2, 11) + "," + candle(3, 12) + "],"
                    + "\"BBB\":[" + candle(1, 20) + "," + candle(2, 22) + "," + candle(3, 24) + "]}}";
            String response = store.command(cmd);
            System.out.println("wickra-feature-store " + FeatureStore.version());
            System.out.println(response);
        }
    }
}
