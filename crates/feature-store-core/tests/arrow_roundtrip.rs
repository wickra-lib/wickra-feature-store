//! Native columnar round-trip (`arrow` feature only): building a golden spec to
//! Parquet and reading it back reproduces the matrix values cell-for-cell,
//! including the `NaN` warmup/label cells. This path is native-only and is not
//! part of the cross-language JSON golden.

#![cfg(feature = "arrow")]

mod common;

use arrow::array::{Array, Float64Array, Int64Array, StringArray};
use feature_store_core::{arrow_out, build, FeatureMatrix};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

/// Read every row group of a Parquet file back into a single flat table:
/// symbols, timestamps, and one `Vec<f64>` per data column.
fn read_parquet(path: &std::path::Path, ncols: usize) -> (Vec<String>, Vec<i64>, Vec<Vec<f64>>) {
    let file = std::fs::File::open(path).expect("open parquet");
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("reader builder")
        .build()
        .expect("reader");

    let mut symbols = Vec::new();
    let mut timestamps = Vec::new();
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); ncols];
    for batch in reader {
        let batch = batch.expect("batch");
        let sym = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("symbol col");
        let ts = batch
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("ts col");
        for i in 0..batch.num_rows() {
            symbols.push(sym.value(i).to_string());
            timestamps.push(ts.value(i));
        }
        for (c, out) in columns.iter_mut().enumerate() {
            let arr = batch
                .column(c + 2)
                .as_any()
                .downcast_ref::<Float64Array>()
                .expect("f64 col");
            for i in 0..batch.num_rows() {
                out.push(arr.value(i));
            }
        }
    }
    (symbols, timestamps, columns)
}

fn assert_roundtrip(name: &str, matrix: &FeatureMatrix) {
    // The in-memory Arrow batch has the documented shape.
    let batch = arrow_out::to_arrow(matrix).expect("to_arrow");
    assert_eq!(batch.num_rows(), matrix.rows, "{name}: arrow row count");
    assert_eq!(
        batch.num_columns(),
        matrix.columns.len() + 2,
        "{name}: arrow column count (symbol + ts + data)"
    );

    let ncols = matrix.columns.len();
    let path = std::env::temp_dir().join(format!("wfs_golden_{name}.parquet"));
    arrow_out::write_parquet(matrix, &path).expect("write parquet");
    let (symbols, timestamps, columns) = read_parquet(&path, ncols);
    let _ = std::fs::remove_file(&path);

    assert_eq!(symbols.len(), matrix.rows, "{name}: parquet row count");
    for (r, id) in matrix.index.iter().enumerate() {
        assert_eq!(symbols[r], id.symbol, "{name}: symbol row {r}");
        assert_eq!(timestamps[r], id.ts, "{name}: ts row {r}");
    }
    for (c, column) in columns.iter().enumerate() {
        for (r, &got) in column.iter().enumerate() {
            let want = matrix.data[r][c];
            if want.is_nan() {
                assert!(got.is_nan(), "{name}: expected NaN at ({r},{c}), got {got}");
            } else {
                assert!(
                    (got - want).abs() < 1e-12,
                    "{name}: value mismatch at ({r},{c}): {got} != {want}"
                );
            }
        }
    }
}

#[test]
fn parquet_roundtrip_reproduces_every_golden_matrix() {
    let data = common::load_data();
    for (name, spec) in common::load_specs() {
        let matrix = build(&data, &spec).unwrap_or_else(|e| panic!("build {name}: {e}"));
        assert_roundtrip(&name, &matrix);
    }
}
