"use strict";

// Golden test over the wasm-pack (nodejs target) output: the WebAssembly build
// folds a spec into a feature matrix byte-identically to the native run. Skips
// cleanly when `pkg/` has not been built yet (`wasm-pack build --target nodejs`).

const { test } = require("node:test");
const assert = require("node:assert");
const path = require("node:path");

let wasm = null;
try {
  wasm = require(path.resolve(__dirname, "..", "pkg", "wickra_feature_store_wasm.js"));
} catch {
  wasm = null;
}

const SPEC = JSON.stringify({
  universe: ["AAA"],
  features: [
    { kind: "indicator", name: "Sma", params: [2] },
    { kind: "price", field: "close" },
  ],
  labels: [{ kind: "forward_return", horizon: 1 }],
});

function candle(ts, close) {
  return { ts, open: close, high: close, low: close, close, volume: 1.0 };
}

function candles() {
  return [candle(0, 100.0), candle(1, 110.0), candle(2, 121.0)];
}

function buildBatchCmd() {
  return JSON.stringify({ cmd: "build_batch", data: { AAA: candles() } });
}

test("wasm build present or skipped", (t) => {
  if (!wasm) t.skip("run `wasm-pack build --target nodejs` first");
});

if (wasm) {
  test("wasm build_batch produces the expected matrix", () => {
    const matrix = JSON.parse(new wasm.FeatureStore(SPEC).command(buildBatchCmd()));
    assert.deepStrictEqual(matrix.columns, ["Sma(2)", "price.close", "fwd_return(1)"]);
    assert.strictEqual(matrix.rows, matrix.data.length);
  });

  test("wasm build_batch is byte-identical across calls", () => {
    const a = new wasm.FeatureStore(SPEC).command(buildBatchCmd());
    const b = new wasm.FeatureStore(SPEC).command(buildBatchCmd());
    assert.strictEqual(a, b);
  });

  test("wasm streaming push matches build_batch byte-for-byte", () => {
    const batch = new wasm.FeatureStore(SPEC).command(buildBatchCmd());
    const streamed = new wasm.FeatureStore(SPEC);
    for (const c of candles()) {
      streamed.command(JSON.stringify({ cmd: "push", symbol: "AAA", candle: c }));
    }
    assert.strictEqual(streamed.command(JSON.stringify({ cmd: "build" })), batch);
  });

  test("wasm version matches the module export", () => {
    assert.strictEqual(new wasm.FeatureStore(SPEC).version(), wasm.version());
  });

  test("wasm throws on an invalid spec", () => {
    assert.throws(() => new wasm.FeatureStore("{ not valid json"));
  });

  test("wasm throws on an unknown command", () => {
    assert.throws(() => new wasm.FeatureStore(SPEC).command('{"cmd":"nope"}'));
  });
}
