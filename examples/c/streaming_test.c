/* Streaming equals batch, through the C ABI's two-call idiom.
 *
 * feature-store-core proves this in Rust, but that says nothing about the
 * boundary a C caller crosses. Every reach behind this ABI asks for the
 * response length first and reads it second, so a command that is not a pure
 * function of its payload runs twice per call — and `push` is exactly that: it
 * mutates the accumulated universe. A double-applied push would fold every
 * candle in twice and nothing outside this test would notice.
 *
 * The dataset here is written inline rather than read from golden/, because C
 * carries no JSON parser and splitting the corpus by hand would test the
 * splitter. What matters is that the same bars reach the core two ways.
 *
 * The spec uses Sma(3), which has a lookback: a column over the raw close reads
 * the same however many times a bar arrived, which is precisely how this class
 * of bug stays invisible.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_feature_store.h"

static const char *SPEC =
    "{\"universe\":[\"AAA\",\"BBB\"],\"features\":["
    "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[3]},"
    "{\"kind\":\"price\",\"field\":\"close\"}]}";

/* Six bars: three per symbol, a rising ramp so Sma(3) is well defined. */
static const char *DATASET =
    "{\"AAA\":["
    "{\"ts\":1,\"open\":10,\"high\":10,\"low\":10,\"close\":10,\"volume\":1},"
    "{\"ts\":2,\"open\":20,\"high\":20,\"low\":20,\"close\":20,\"volume\":1},"
    "{\"ts\":3,\"open\":30,\"high\":30,\"low\":30,\"close\":30,\"volume\":1}],"
    "\"BBB\":["
    "{\"ts\":1,\"open\":40,\"high\":40,\"low\":40,\"close\":40,\"volume\":1},"
    "{\"ts\":2,\"open\":50,\"high\":50,\"low\":50,\"close\":50,\"volume\":1},"
    "{\"ts\":3,\"open\":60,\"high\":60,\"low\":60,\"close\":60,\"volume\":1}]}";

static const char *PUSHES[] = {
    "{\"cmd\":\"push\",\"symbol\":\"AAA\",\"candle\":{\"ts\":1,\"open\":10,\"high\":10,\"low\":10,\"close\":10,\"volume\":1}}",
    "{\"cmd\":\"push\",\"symbol\":\"AAA\",\"candle\":{\"ts\":2,\"open\":20,\"high\":20,\"low\":20,\"close\":20,\"volume\":1}}",
    "{\"cmd\":\"push\",\"symbol\":\"AAA\",\"candle\":{\"ts\":3,\"open\":30,\"high\":30,\"low\":30,\"close\":30,\"volume\":1}}",
    "{\"cmd\":\"push\",\"symbol\":\"BBB\",\"candle\":{\"ts\":1,\"open\":40,\"high\":40,\"low\":40,\"close\":40,\"volume\":1}}",
    "{\"cmd\":\"push\",\"symbol\":\"BBB\",\"candle\":{\"ts\":2,\"open\":50,\"high\":50,\"low\":50,\"close\":50,\"volume\":1}}",
    "{\"cmd\":\"push\",\"symbol\":\"BBB\",\"candle\":{\"ts\":3,\"open\":60,\"high\":60,\"low\":60,\"close\":60,\"volume\":1}}",
};
static const size_t PUSH_COUNT = sizeof(PUSHES) / sizeof(PUSHES[0]);

/* Run one command through the documented two-call idiom. Caller frees. */
static char *run(WickraFeatureStore *store, const char *cmd) {
    int32_t len = wickra_feature_store_command(store, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", (int)len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    int32_t written = wickra_feature_store_command(store, cmd, buf, (size_t)len + 1);
    if (written != len) {
        fprintf(stderr, "second call returned %d, first said %d\n", (int)written, (int)len);
        free(buf);
        return NULL;
    }
    return buf;
}

int main(void) {
    char command[4096];
    int n = snprintf(command, sizeof(command), "{\"cmd\":\"build_batch\",\"data\":%s}", DATASET);
    if (n < 0 || (size_t)n >= sizeof(command)) {
        fprintf(stderr, "build_batch command did not fit\n");
        return 1;
    }

    WickraFeatureStore *batch_store = wickra_feature_store_new(SPEC);
    if (!batch_store) {
        fprintf(stderr, "failed to build the batch store\n");
        return 1;
    }
    char *batch = run(batch_store, command);
    wickra_feature_store_free(batch_store);
    if (!batch) {
        return 1;
    }

    WickraFeatureStore *stream_store = wickra_feature_store_new(SPEC);
    if (!stream_store) {
        fprintf(stderr, "failed to build the streaming store\n");
        free(batch);
        return 1;
    }
    for (size_t i = 0; i < PUSH_COUNT; i++) {
        char *ack = run(stream_store, PUSHES[i]);
        if (!ack) {
            wickra_feature_store_free(stream_store);
            free(batch);
            return 1;
        }
        free(ack);
    }
    char *streamed = run(stream_store, "{\"cmd\":\"build\"}");
    wickra_feature_store_free(stream_store);
    if (!streamed) {
        free(batch);
        return 1;
    }

    int equal = strcmp(batch, streamed) == 0;
    printf("batch    : %s\n", batch);
    printf("streaming: %s\n", streamed);

    /* Sma(3) over 10, 20, 30 is 20 and over 40, 50, 60 is 50. Pushed twice each
     * they would be the means of 20, 30, 30 and 50, 60, 60 instead, so these
     * values are what tells the two apart. */
    int has_expected = strstr(batch, "[20,30]") != NULL && strstr(batch, "[50,60]") != NULL;

    free(batch);
    free(streamed);

    if (!equal) {
        fprintf(stderr, "streaming != batch\n");
        return 1;
    }
    if (!has_expected) {
        fprintf(stderr, "expected Sma(3) of 20 and 50 from three bars each\n");
        return 1;
    }
    printf("streaming equals batch\n");
    return 0;
}
