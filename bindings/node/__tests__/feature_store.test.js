"use strict";

const { test } = require("node:test");
const assert = require("node:assert");
const { FeatureStore } = require("../index.js");

const SPEC = {
  universe: ["AAA"],
  features: [
    { kind: "indicator", name: "Sma", params: [2] },
    { kind: "price", field: "close" },
  ],
  labels: [{ kind: "forward_return", horizon: 1 }],
};

function candle(ts, close) {
  return { ts, open: close, high: close, low: close, close, volume: 1.0 };
}

function candles() {
  return [candle(0, 100.0), candle(1, 110.0), candle(2, 121.0)];
}

function buildBatchCmd() {
  return JSON.stringify({ cmd: "build_batch", data: { AAA: candles() } });
}

test("build_batch returns the expected feature matrix", () => {
  const store = new FeatureStore(JSON.stringify(SPEC));
  const matrix = JSON.parse(store.command(buildBatchCmd()));
  assert.deepStrictEqual(matrix.columns, ["Sma(2)", "price.close", "fwd_return(1)"]);
  assert.strictEqual(matrix.rows, matrix.data.length);
  assert.strictEqual(matrix.data.length, matrix.index.length);
});

test("streaming push matches build_batch byte-for-byte", () => {
  const batch = new FeatureStore(JSON.stringify(SPEC)).command(buildBatchCmd());

  const streamed = new FeatureStore(JSON.stringify(SPEC));
  for (const c of candles()) {
    streamed.command(JSON.stringify({ cmd: "push", symbol: "AAA", candle: c }));
  }
  const built = streamed.command(JSON.stringify({ cmd: "build" }));
  assert.strictEqual(built, batch);
});

test("an unknown command throws", () => {
  const store = new FeatureStore(JSON.stringify(SPEC));
  assert.throws(() => store.command(JSON.stringify({ cmd: "nope" })));
});

test("an invalid spec throws", () => {
  assert.throws(() => new FeatureStore("{ not valid json"));
});

test("version is a string", () => {
  const store = new FeatureStore(JSON.stringify(SPEC));
  assert.strictEqual(typeof store.version(), "string");
});

module.exports = { SPEC, candle, candles, buildBatchCmd };
