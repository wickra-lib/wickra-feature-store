//! Columnar Arrow / Parquet output — native only, behind the `arrow` feature.
//!
//! The schema is `symbol: Utf8`, `ts: Int64`, then one `Float64` column per
//! feature and label key (in matrix column order). `NaN` is stored natively (no
//! `null` mapping), so this path is intentionally *not* part of the byte-exact
//! cross-language golden — only native round-trip tests cover it.

use crate::error::{Error, Result};
use crate::matrix::FeatureMatrix;
use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Convert a feature matrix into an Arrow [`RecordBatch`].
///
/// # Errors
/// Returns [`Error::Output`] if the arrays do not form a valid batch.
pub fn to_arrow(matrix: &FeatureMatrix) -> Result<RecordBatch> {
    let mut fields: Vec<Field> = vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("ts", DataType::Int64, false),
    ];
    for col in &matrix.columns {
        fields.push(Field::new(col, DataType::Float64, true));
    }
    let schema = Arc::new(Schema::new(fields));

    let symbols = StringArray::from(
        matrix
            .index
            .iter()
            .map(|id| id.symbol.clone())
            .collect::<Vec<_>>(),
    );
    let timestamps = Int64Array::from(matrix.index.iter().map(|id| id.ts).collect::<Vec<_>>());

    let mut columns: Vec<arrow::array::ArrayRef> = vec![Arc::new(symbols), Arc::new(timestamps)];
    for c in 0..matrix.columns.len() {
        let values: Vec<f64> = matrix.data.iter().map(|row| row[c]).collect();
        columns.push(Arc::new(Float64Array::from(values)));
    }

    RecordBatch::try_new(schema, columns).map_err(|e| Error::Output(e.to_string()))
}

/// Write a feature matrix to a Parquet file.
///
/// # Errors
/// Returns [`Error::Output`] if the file cannot be created or the write fails.
pub fn write_parquet(matrix: &FeatureMatrix, path: &std::path::Path) -> Result<()> {
    use parquet::arrow::ArrowWriter;

    let batch = to_arrow(matrix)?;
    let file = std::fs::File::create(path).map_err(|e| Error::Output(e.to_string()))?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)
        .map_err(|e| Error::Output(e.to_string()))?;
    writer
        .write(&batch)
        .map_err(|e| Error::Output(e.to_string()))?;
    writer.close().map_err(|e| Error::Output(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::RowId;

    fn matrix() -> FeatureMatrix {
        let mut m = FeatureMatrix::new(vec!["price.close".into(), "fwd_return(1)".into()]);
        m.push_row(
            RowId {
                symbol: "AAA".into(),
                ts: 1,
            },
            vec![100.0, 0.1],
        );
        m.push_row(
            RowId {
                symbol: "AAA".into(),
                ts: 2,
            },
            vec![110.0, f64::NAN],
        );
        m
    }

    #[test]
    fn arrow_batch_has_symbol_ts_and_feature_columns() {
        let batch = to_arrow(&matrix()).unwrap();
        assert_eq!(batch.num_columns(), 4);
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.schema().field(0).name(), "symbol");
        assert_eq!(batch.schema().field(1).name(), "ts");
    }

    #[test]
    fn parquet_round_trips() {
        use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

        let dir = std::env::temp_dir();
        let path = dir.join("wickra_fs_arrow_out_test.parquet");
        write_parquet(&matrix(), &path).unwrap();
        let file = std::fs::File::open(&path).unwrap();
        let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let batch = reader.next().unwrap().unwrap();
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 4);
        let _ = std::fs::remove_file(&path);
    }
}
