"use strict";

// Parity guard: the Node binding must expose the full public surface, so an
// export dropped in a refactor fails loudly here.

const { test } = require("node:test");
const assert = require("node:assert");
const wickra = require("../index.js");

test("module exposes FeatureStore", () => {
  assert.strictEqual(typeof wickra.FeatureStore, "function");
});

test("FeatureStore exposes command and version", () => {
  for (const name of ["command", "version"]) {
    assert.strictEqual(
      typeof wickra.FeatureStore.prototype[name],
      "function",
      `FeatureStore is missing ${name}`,
    );
  }
});

test("FeatureStore surface is exactly {command, version}", () => {
  const methods = Object.getOwnPropertyNames(wickra.FeatureStore.prototype)
    .filter((name) => name !== "constructor")
    .sort();
  assert.deepStrictEqual(methods, ["command", "version"]);
});
