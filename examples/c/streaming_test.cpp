// Streaming equals batch, through the C++ hull.
//
// The hull hides the two-call length protocol, which is exactly why it needs
// its own test: `command` asks the ABI for a length and then reads, so a
// mutating command sent by a C++ caller would run twice if the ABI did not
// carry the produced response between the calls. The golden corpus cannot see
// that -- it only ever sends {"cmd":"build_batch"}, a pure function of its
// payload -- while `push` mutates the accumulated universe.
//
// The dataset is inline rather than read from golden/, because splitting the
// corpus by hand here would test the splitter. What matters is that the same
// bars reach the core two ways.
//
// The spec uses Sma(3), which has a lookback: a column over the raw close reads
// the same however many times a bar arrived, which is how this class of bug
// stays invisible.
#include <exception>
#include <iostream>
#include <string>
#include <vector>

#include "wickra_feature_store.hpp"

namespace {
const char *SPEC =
    R"({"universe":["AAA","BBB"],"features":[)"
    R"({"kind":"indicator","name":"Sma","params":[3]},)"
    R"({"kind":"price","field":"close"}]})";

const char *DATASET =
    R"({"AAA":[)"
    R"({"ts":1,"open":10,"high":10,"low":10,"close":10,"volume":1},)"
    R"({"ts":2,"open":20,"high":20,"low":20,"close":20,"volume":1},)"
    R"({"ts":3,"open":30,"high":30,"low":30,"close":30,"volume":1}],)"
    R"("BBB":[)"
    R"({"ts":1,"open":40,"high":40,"low":40,"close":40,"volume":1},)"
    R"({"ts":2,"open":50,"high":50,"low":50,"close":50,"volume":1},)"
    R"({"ts":3,"open":60,"high":60,"low":60,"close":60,"volume":1}]})";

const std::vector<std::string> PUSHES = {
    R"({"cmd":"push","symbol":"AAA","candle":{"ts":1,"open":10,"high":10,"low":10,"close":10,"volume":1}})",
    R"({"cmd":"push","symbol":"AAA","candle":{"ts":2,"open":20,"high":20,"low":20,"close":20,"volume":1}})",
    R"({"cmd":"push","symbol":"AAA","candle":{"ts":3,"open":30,"high":30,"low":30,"close":30,"volume":1}})",
    R"({"cmd":"push","symbol":"BBB","candle":{"ts":1,"open":40,"high":40,"low":40,"close":40,"volume":1}})",
    R"({"cmd":"push","symbol":"BBB","candle":{"ts":2,"open":50,"high":50,"low":50,"close":50,"volume":1}})",
    R"({"cmd":"push","symbol":"BBB","candle":{"ts":3,"open":60,"high":60,"low":60,"close":60,"volume":1}})",
};
}  // namespace

int main() {
    try {
        wickra::FeatureStore batchStore(SPEC);
        const std::string batch =
            batchStore.command(std::string(R"({"cmd":"build_batch","data":)") + DATASET + "}");

        wickra::FeatureStore streaming(SPEC);
        for (const std::string &push : PUSHES) {
            streaming.command(push);
        }
        const std::string streamed = streaming.command(R"({"cmd":"build"})");

        std::cout << "batch    : " << batch << "\n";
        std::cout << "streaming: " << streamed << "\n";

        if (batch != streamed) {
            std::cerr << "streaming != batch\n";
            return 1;
        }
        // Sma(3) over 10, 20, 30 is 20 and over 40, 50, 60 is 50. Pushed twice
        // each they would be the means of 20, 30, 30 and 50, 60, 60 instead, so
        // these cells are what tells the two apart.
        if (batch.find("[20,30]") == std::string::npos ||
            batch.find("[50,60]") == std::string::npos) {
            std::cerr << "expected Sma(3) of 20 and 50 from three bars each\n";
            return 1;
        }
        std::cout << "streaming equals batch\n";
    } catch (const std::exception &e) {
        std::cerr << "failed: " << e.what() << "\n";
        return 1;
    }
    return 0;
}
