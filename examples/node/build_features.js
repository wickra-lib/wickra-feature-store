// A runnable Node.js example: build a feature matrix through the binding.
//
//   ( cd bindings/node && npm install && npm run build )
//   ( cd examples/node && npm install && node build_features.js )

"use strict";

const { FeatureStore } = require("wickra-feature-store");

const SPEC = JSON.stringify({
  universe: ["AAA", "BBB"],
  features: [
    { kind: "indicator", name: "Sma", params: [2] },
    { kind: "price", field: "close" },
  ],
  labels: [{ kind: "forward_return", horizon: 1 }],
});

const candle = (time, close) => ({
  time,
  open: close,
  high: close,
  low: close,
  close,
  volume: 1.0,
});

const series = (closes) => closes.map((c, i) => candle(i + 1, c));

const store = new FeatureStore(SPEC);
const response = store.command(
  JSON.stringify({
    cmd: "build_batch",
    data: {
      AAA: series([10.0, 11.0, 12.0]),
      BBB: series([20.0, 22.0, 24.0]),
    },
  }),
);
const matrix = JSON.parse(response);

console.log("wickra-feature-store", store.version());
console.log("columns:", matrix.columns);
console.log("rows:", matrix.rows);
console.log(response);
