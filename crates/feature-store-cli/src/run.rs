//! Load the spec and universe, build the feature matrix, and emit it.

use crate::args::{Args, Format};
use feature_store_core::{build, Candle, FeatureSpec, OutputFormat};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::Path;

/// A candle in its JSON input form (`ts` is the timestamp; the core `Candle`
/// calls the field `time`). Mirrors the boundary type in `feature-store-core`.
#[derive(Deserialize)]
struct CandleInput {
    ts: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl From<CandleInput> for Candle {
    fn from(c: CandleInput) -> Self {
        Candle {
            time: c.ts,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }
}

/// Load the inputs, build the matrix and write the output where requested.
///
/// # Errors
/// Returns a human-readable message on any I/O, parse, spec or build failure.
pub fn run(args: &Args) -> Result<(), String> {
    let mut spec = load_spec(&args.spec)?;
    if let Some(window) = args.window {
        spec.window = Some(window);
    }
    spec.validate().map_err(|e| e.to_string())?;

    let data = if args.stdin {
        load_stdin()?
    } else if let Some(dir) = &args.data {
        load_data_dir(dir)?
    } else {
        // clap's required ArgGroup guarantees one of the two is present.
        return Err("no data source (pass --data or --stdin)".to_string());
    };

    let matrix = build(&data, &spec).map_err(|e| e.to_string())?;
    let format = resolve_format(args.format, spec.output);

    match format {
        OutputFormat::Json => emit_text(&with_newline(&matrix.to_json()), args.out.as_deref()),
        OutputFormat::Csv => emit_text(&matrix.to_csv(), args.out.as_deref()),
        OutputFormat::Arrow | OutputFormat::Parquet => {
            write_columnar(&matrix, format, args.out.as_deref())
        }
    }
}

/// Read and parse a spec file, choosing JSON or TOML by extension.
fn load_spec(path: &Path) -> Result<FeatureSpec, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("read spec {}: {e}", path.display()))?;
    let is_toml = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
    let parsed = if is_toml {
        FeatureSpec::from_toml(&content)
    } else {
        FeatureSpec::from_json(&content)
    };
    parsed.map_err(|e| e.to_string())
}

/// Load a universe from a directory of `<SYMBOL>.csv` files.
fn load_data_dir(dir: &Path) -> Result<BTreeMap<String, Vec<Candle>>, String> {
    let mut data = BTreeMap::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read dir {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }
        let symbol = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("bad file name: {}", path.display()))?
            .to_string();
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        data.insert(symbol, parse_csv(&content)?);
    }
    Ok(data)
}

/// Load a universe as a JSON dataset (`{"SYMBOL": [candle, ...]}`) from stdin.
fn load_stdin() -> Result<BTreeMap<String, Vec<Candle>>, String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| e.to_string())?;
    let raw: BTreeMap<String, Vec<CandleInput>> =
        serde_json::from_str(&buf).map_err(|e| format!("parse stdin dataset: {e}"))?;
    Ok(raw
        .into_iter()
        .map(|(symbol, candles)| (symbol, candles.into_iter().map(Into::into).collect()))
        .collect())
}

/// Parse OHLCV rows (`ts,open,high,low,close,volume`) into candles; a
/// non-numeric first row is treated as a header and skipped.
fn parse_csv(content: &str) -> Result<Vec<Candle>, String> {
    let mut candles = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        if cols.len() < 6 {
            return Err(format!(
                "CSV line {}: expected 6 columns, got {}",
                idx + 1,
                cols.len()
            ));
        }
        let time = match cols[0].parse::<i64>() {
            Ok(t) => t,
            Err(_) if idx == 0 => continue, // header row
            Err(e) => return Err(format!("CSV line {}: bad timestamp: {e}", idx + 1)),
        };
        let field = |i: usize, name: &str| {
            cols[i]
                .parse::<f64>()
                .map_err(|e| format!("CSV line {}: {name}: {e}", idx + 1))
        };
        candles.push(Candle {
            time,
            open: field(1, "open")?,
            high: field(2, "high")?,
            low: field(3, "low")?,
            close: field(4, "close")?,
            volume: field(5, "volume")?,
        });
    }
    Ok(candles)
}

/// The effective output format: the `--format` override, else the spec default.
fn resolve_format(requested: Option<Format>, spec_default: OutputFormat) -> OutputFormat {
    match requested {
        Some(Format::Json) => OutputFormat::Json,
        Some(Format::Csv) => OutputFormat::Csv,
        Some(Format::Arrow) => OutputFormat::Arrow,
        Some(Format::Parquet) => OutputFormat::Parquet,
        None => spec_default,
    }
}

/// A copy of `text` with a single trailing newline.
fn with_newline(text: &str) -> String {
    let mut s = text.to_string();
    s.push('\n');
    s
}

/// Write text output to `out` if given, else to standard output.
fn emit_text(text: &str, out: Option<&Path>) -> Result<(), String> {
    if let Some(path) = out {
        std::fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
    } else {
        print!("{text}");
        Ok(())
    }
}

/// Write a columnar (Arrow IPC / Parquet) matrix to a file. Only available when
/// the crate is built with the `arrow` feature.
#[cfg(feature = "arrow")]
fn write_columnar(
    matrix: &feature_store_core::FeatureMatrix,
    format: OutputFormat,
    out: Option<&Path>,
) -> Result<(), String> {
    let path = out.ok_or("--out is required for arrow/parquet output")?;
    match format {
        OutputFormat::Parquet => {
            feature_store_core::arrow_out::write_parquet(matrix, path).map_err(|e| e.to_string())
        }
        OutputFormat::Arrow => write_arrow_ipc(matrix, path),
        OutputFormat::Json | OutputFormat::Csv => unreachable!("text formats handled by caller"),
    }
}

#[cfg(not(feature = "arrow"))]
fn write_columnar(
    _matrix: &feature_store_core::FeatureMatrix,
    _format: OutputFormat,
    _out: Option<&Path>,
) -> Result<(), String> {
    Err("this binary was built without arrow/parquet support; rebuild with --features arrow".into())
}

/// Write the matrix as an Arrow IPC file.
#[cfg(feature = "arrow")]
fn write_arrow_ipc(matrix: &feature_store_core::FeatureMatrix, path: &Path) -> Result<(), String> {
    use arrow::ipc::writer::FileWriter;

    let batch = feature_store_core::arrow_out::to_arrow(matrix).map_err(|e| e.to_string())?;
    let file =
        std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut writer = FileWriter::try_new(file, &batch.schema()).map_err(|e| e.to_string())?;
    writer.write(&batch).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_csv_with_a_header() {
        let csv = "ts,open,high,low,close,volume\n1,10,11,9,10.5,100\n2,10.5,12,10,11,200\n";
        let candles = parse_csv(csv).unwrap();
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].time, 1);
        assert!((candles[1].close - 11.0).abs() < 1e-9);
    }

    #[test]
    fn parse_csv_rejects_a_short_row() {
        assert!(parse_csv("1,2,3\n").is_err());
    }

    #[test]
    fn resolve_format_prefers_the_override() {
        assert_eq!(
            resolve_format(Some(Format::Csv), OutputFormat::Json),
            OutputFormat::Csv
        );
        assert_eq!(
            resolve_format(None, OutputFormat::Parquet),
            OutputFormat::Parquet
        );
    }

    #[test]
    fn candle_input_maps_ts_to_time() {
        let c: Candle = CandleInput {
            ts: 7,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
        }
        .into();
        assert_eq!(c.time, 7);
        assert!((c.close - 1.5).abs() < 1e-12);
    }

    #[test]
    fn json_output_ends_with_a_newline() {
        assert!(with_newline("{}").ends_with("}\n"));
    }
}
