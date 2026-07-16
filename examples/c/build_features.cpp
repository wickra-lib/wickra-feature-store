// A minimal C++ example: build a feature matrix through the wickra-feature-store
// C ABI.
#include <cstddef>
#include <iostream>
#include <string>
#include <vector>

#include "wickra_feature_store.h"

namespace {
const char *SPEC =
    R"({"universe":["AAA","BBB"],"features":[)"
    R"({"kind":"indicator","name":"Sma","params":[2]},)"
    R"({"kind":"price","field":"close"}],)"
    R"("labels":[{"kind":"forward_return","horizon":1}]})";

const char *CMD =
    R"({"cmd":"build_batch","data":{)"
    R"("AAA":[)"
    R"({"time":1,"open":10,"high":10,"low":10,"close":10,"volume":1},)"
    R"({"time":2,"open":11,"high":11,"low":11,"close":11,"volume":1},)"
    R"({"time":3,"open":12,"high":12,"low":12,"close":12,"volume":1}],)"
    R"("BBB":[)"
    R"({"time":1,"open":20,"high":20,"low":20,"close":20,"volume":1},)"
    R"({"time":2,"open":22,"high":22,"low":22,"close":22,"volume":1},)"
    R"({"time":3,"open":24,"high":24,"low":24,"close":24,"volume":1}]}})";
}  // namespace

int main() {
    WickraFeatureStore *store = wickra_feature_store_new(SPEC);
    if (store == nullptr) {
        std::cerr << "failed to build feature store\n";
        return 1;
    }

    int len = wickra_feature_store_command(store, CMD, nullptr, 0);
    if (len < 0) {
        std::cerr << "command failed: code " << len << "\n";
        wickra_feature_store_free(store);
        return 1;
    }
    std::vector<char> buf(static_cast<std::size_t>(len) + 1);
    wickra_feature_store_command(store, CMD, buf.data(),
                                 static_cast<std::size_t>(buf.size()));

    std::cout << "wickra-feature-store " << wickra_feature_store_version() << "\n";
    std::cout << "matrix: " << std::string(buf.data()) << "\n";

    wickra_feature_store_free(store);
    return 0;
}
