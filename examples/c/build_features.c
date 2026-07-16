/* A minimal C example: build a feature matrix through the wickra-feature-store
 * C ABI. */
#include <stdio.h>
#include <stdlib.h>

#include "wickra_feature_store.h"

static const char *SPEC =
    "{\"universe\":[\"AAA\",\"BBB\"],\"features\":["
    "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[2]},"
    "{\"kind\":\"price\",\"field\":\"close\"}],"
    "\"labels\":[{\"kind\":\"forward_return\",\"horizon\":1}]}";

static const char *CMD =
    "{\"cmd\":\"build_batch\",\"data\":{"
    "\"AAA\":["
    "{\"time\":1,\"open\":10,\"high\":10,\"low\":10,\"close\":10,\"volume\":1},"
    "{\"time\":2,\"open\":11,\"high\":11,\"low\":11,\"close\":11,\"volume\":1},"
    "{\"time\":3,\"open\":12,\"high\":12,\"low\":12,\"close\":12,\"volume\":1}],"
    "\"BBB\":["
    "{\"time\":1,\"open\":20,\"high\":20,\"low\":20,\"close\":20,\"volume\":1},"
    "{\"time\":2,\"open\":22,\"high\":22,\"low\":22,\"close\":22,\"volume\":1},"
    "{\"time\":3,\"open\":24,\"high\":24,\"low\":24,\"close\":24,\"volume\":1}]}}";

int main(void) {
    WickraFeatureStore *store = wickra_feature_store_new(SPEC);
    if (!store) {
        fprintf(stderr, "failed to build feature store\n");
        return 1;
    }

    /* Length-out protocol: learn the length, then read into a caller buffer. */
    int len = wickra_feature_store_command(store, CMD, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        wickra_feature_store_free(store);
        return 1;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        wickra_feature_store_free(store);
        return 1;
    }
    wickra_feature_store_command(store, CMD, buf, (size_t)len + 1);

    printf("wickra-feature-store %s\n", wickra_feature_store_version());
    printf("matrix: %s\n", buf);

    free(buf);
    wickra_feature_store_free(store);
    return 0;
}
