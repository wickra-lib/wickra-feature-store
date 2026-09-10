// A minimal C++ example: build a feature matrix over a two-symbol universe.
//
// This goes through `wickra_feature_store.hpp`, the C++ hull shipped beside the
// C header, because that hull is what a C++ caller is meant to use: it owns and
// frees the handle, runs the two-call length protocol behind
// `wickra_feature_store_command` for you, and turns a refusal into an exception
// rather than a negative integer that is easy to ignore. Calling the C
// functions directly from C++ works too -- `build_features.c` shows that -- but
// then the hull would be shipped with only `streaming_test.cpp` building it.
#include <exception>
#include <iostream>
#include <string>

#include "wickra_feature_store.hpp"

namespace {
const char *SPEC =
    R"({"universe":["AAA","BBB"],"features":[)"
    R"({"kind":"indicator","name":"Sma","params":[2]},)"
    R"({"kind":"price","field":"close"}],)"
    R"("labels":[{"kind":"forward_return","horizon":1}]})";

const char *CMD =
    R"({"cmd":"build_batch","data":{)"
    R"("AAA":[)"
    R"({"ts":1,"open":10,"high":10,"low":10,"close":10,"volume":1},)"
    R"({"ts":2,"open":11,"high":11,"low":11,"close":11,"volume":1},)"
    R"({"ts":3,"open":12,"high":12,"low":12,"close":12,"volume":1}],)"
    R"("BBB":[)"
    R"({"ts":1,"open":20,"high":20,"low":20,"close":20,"volume":1},)"
    R"({"ts":2,"open":22,"high":22,"low":22,"close":22,"volume":1},)"
    R"({"ts":3,"open":24,"high":24,"low":24,"close":24,"volume":1}]}})";
}  // namespace

int main() {
    try {
        wickra::FeatureStore store(SPEC);
        const std::string matrix = store.command(CMD);

        std::cout << "wickra-feature-store " << wickra::FeatureStore::version() << "\n";
        std::cout << "matrix: " << matrix << "\n";
    } catch (const std::exception &e) {
        std::cerr << "failed: " << e.what() << "\n";
        return 1;
    }
    return 0;
}
