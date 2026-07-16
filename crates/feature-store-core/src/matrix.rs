//! The feature matrix output type and its deterministic serialization.
//!
//! `serde_json` cannot render `NaN`/`inf` as a JSON number, and the byte-exact
//! golden corpus demands one canonical rendering, so cells are serialized by
//! hand: non-finite values become JSON `null`, finite values are rounded with
//! [`round_to`] and rendered with the shortest round-trip float form.

use serde::{Deserialize, Serialize};

/// Round a value to a fixed grid (`1e-8`) so every language rounds identically.
/// Non-finite values pass through unchanged (they serialize to `null`).
#[must_use]
pub fn round_to(x: f64, precision: f64) -> f64 {
    if x.is_finite() {
        (x / precision).round() * precision
    } else {
        x
    }
}

/// Render one cell: `null` for non-finite values, otherwise the rounded number.
pub(crate) fn serialize_cell(x: f64) -> String {
    if x.is_finite() {
        round_to(x, 1e-8).to_string()
    } else {
        "null".to_string()
    }
}

/// The identity of one emitted row: its symbol and bar timestamp.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RowId {
    /// The symbol the row belongs to.
    pub symbol: String,
    /// The bar timestamp (as carried by the input candle).
    pub ts: i64,
}

/// A materialized feature matrix: fixed column order (features then labels),
/// one row per emitted bar in emission order.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FeatureMatrix {
    /// Canonical column keys: feature keys (spec order) followed by label keys.
    pub columns: Vec<String>,
    /// Per-row `(symbol, ts)` identity, in emission order.
    pub index: Vec<RowId>,
    /// Row-major cells; `data[r].len() == columns.len()`; `NaN` is allowed.
    pub data: Vec<Vec<f64>>,
    /// The number of rows (`== data.len()`).
    pub rows: usize,
}

impl FeatureMatrix {
    /// An empty matrix with the given column keys.
    #[must_use]
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            index: Vec::new(),
            data: Vec::new(),
            rows: 0,
        }
    }

    /// Append one row and its identity. `cells.len()` must equal the column
    /// count; callers guarantee this by construction.
    pub fn push_row(&mut self, id: RowId, cells: Vec<f64>) {
        self.index.push(id);
        self.data.push(cells);
        self.rows = self.data.len();
    }

    /// Render as canonical JSON, non-finite cells as `null`.
    #[must_use]
    pub fn to_json(&self) -> String {
        let columns = serde_json::to_string(&self.columns).unwrap_or_else(|_| "[]".to_string());
        let index = serde_json::to_string(&self.index).unwrap_or_else(|_| "[]".to_string());
        let mut data = String::from("[");
        for (r, row) in self.data.iter().enumerate() {
            if r > 0 {
                data.push(',');
            }
            data.push('[');
            for (c, &cell) in row.iter().enumerate() {
                if c > 0 {
                    data.push(',');
                }
                data.push_str(&serialize_cell(cell));
            }
            data.push(']');
        }
        data.push(']');
        format!(
            "{{\"columns\":{columns},\"index\":{index},\"data\":{data},\"rows\":{}}}",
            self.rows
        )
    }

    /// Render as CSV: header `symbol,ts,<col>,...`, non-finite cells empty.
    #[must_use]
    pub fn to_csv(&self) -> String {
        let mut out = String::from("symbol,ts");
        for col in &self.columns {
            out.push(',');
            out.push_str(col);
        }
        out.push('\n');
        for (id, row) in self.index.iter().zip(&self.data) {
            out.push_str(&id.symbol);
            out.push(',');
            out.push_str(&id.ts.to_string());
            for &cell in row {
                out.push(',');
                if cell.is_finite() {
                    out.push_str(&round_to(cell, 1e-8).to_string());
                }
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_to_grids() {
        assert!((round_to(0.021_000_004, 1e-8) - 0.021).abs() < 1e-9);
        assert!(round_to(f64::NAN, 1e-8).is_nan());
    }

    #[test]
    fn nan_serializes_as_null() {
        let mut m = FeatureMatrix::new(vec!["a".into(), "b".into()]);
        m.push_row(
            RowId {
                symbol: "AAA".into(),
                ts: 1,
            },
            vec![1.5, f64::NAN],
        );
        let json = m.to_json();
        assert!(json.contains("[1.5,null]"), "{json}");
        assert!(json.contains("\"rows\":1"));
    }

    #[test]
    fn csv_has_header_and_empty_nan() {
        let mut m = FeatureMatrix::new(vec!["a".into()]);
        m.push_row(
            RowId {
                symbol: "AAA".into(),
                ts: 7,
            },
            vec![f64::NAN],
        );
        let csv = m.to_csv();
        assert_eq!(csv, "symbol,ts,a\nAAA,7,\n");
    }
}
