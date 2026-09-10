"use strict";

// The cross-language golden invariant seen from Node: the same command yields
// byte-identical output across calls, and — once the P-FS-4 fixtures land — the
// blessed golden re-matches byte-for-byte. The response bytes are what every
// other binding produces too.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { FeatureStore } = require("../index.js");
const { SPEC, buildBatchCmd } = require("./feature_store.test.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");

test("build_batch is byte-identical across calls", () => {
  const a = new FeatureStore(JSON.stringify(SPEC)).command(buildBatchCmd());
  const b = new FeatureStore(JSON.stringify(SPEC)).command(buildBatchCmd());
  assert.strictEqual(a, b);
});

test(
  "build_batch matches the committed golden byte-for-byte",
  { skip: !fs.existsSync(GOLDEN) ? "golden fixtures not present yet" : false },
  () => {
    // A spec named `feeds_*` is folded over the dataset carrying one side feed
    // per family; every other spec over the candle-only one. Sending the
    // candle-only dataset to a fed spec is refused by name -- that refusal is
    // the point of the feed check, so picking the right dataset is part of
    // speaking the protocol correctly.
    const load = (name) =>
      JSON.parse(fs.readFileSync(path.join(GOLDEN, name), "utf8"));
    const candlesOnly = load("data.json");
    const fed = load("data-feeds.json");
    const specs = fs
      .readdirSync(path.join(GOLDEN, "specs"))
      .filter((f) => f.endsWith(".json"))
      .sort();
    for (const specFile of specs) {
      const spec = fs.readFileSync(path.join(GOLDEN, "specs", specFile), "utf8");
      const expected = fs
        .readFileSync(path.join(GOLDEN, "expected", specFile), "utf8")
        .trim();
      const dataset = specFile.startsWith("feeds_") ? fed : candlesOnly;
      const got = new FeatureStore(spec).command(
        JSON.stringify({ cmd: "build_batch", data: dataset }),
      );
      assert.strictEqual(got, expected, `${specFile} must be byte-identical to the Rust golden`);
    }
  },
);
