//! CLI argument parsing.

use clap::{ArgGroup, Parser, ValueEnum};
use std::path::PathBuf;

/// Fold a `FeatureSpec` over a universe of candles and emit the feature matrix.
#[derive(Parser, Debug)]
#[command(name = "wickra-feature-store", version, about)]
#[command(group(ArgGroup::new("source").required(true).args(["data", "stdin"])))]
pub struct Args {
    /// Path to the feature spec (JSON or TOML, chosen by file extension).
    #[arg(long)]
    pub spec: PathBuf,

    /// Directory of per-symbol CSV candle files (`<SYMBOL>.csv`).
    #[arg(long)]
    pub data: Option<PathBuf>,

    /// Read the universe as a JSON dataset (`{"SYMBOL": [candle, ...]}`) from
    /// standard input instead.
    #[arg(long)]
    pub stdin: bool,

    /// Output format. Defaults to the spec's `output` field when omitted.
    #[arg(long, value_enum)]
    pub format: Option<Format>,

    /// Write output to this file instead of standard output. Required for the
    /// binary `arrow`/`parquet` formats.
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Override the spec's trailing row window (keep only the last `N` rows per
    /// symbol).
    #[arg(long)]
    pub window: Option<usize>,
}

/// The matrix output format requested on the command line.
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum Format {
    /// Canonical JSON.
    Json,
    /// CSV.
    Csv,
    /// Apache Arrow IPC file (native only, requires the `arrow` build feature).
    Arrow,
    /// Apache Parquet (native only, requires the `arrow` build feature).
    Parquet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn arg_config_is_valid() {
        Args::command().debug_assert();
    }

    #[test]
    fn parses_a_data_source() {
        let args =
            Args::try_parse_from(["wickra-feature-store", "--spec", "s.json", "--data", "d"])
                .unwrap();
        assert_eq!(args.data, Some(PathBuf::from("d")));
        assert!(args.format.is_none());
        assert!(!args.stdin);
    }

    #[test]
    fn data_and_stdin_conflict() {
        assert!(Args::try_parse_from([
            "wickra-feature-store",
            "--spec",
            "s.json",
            "--data",
            "d",
            "--stdin"
        ])
        .is_err());
    }

    #[test]
    fn a_source_is_required() {
        assert!(Args::try_parse_from(["wickra-feature-store", "--spec", "s.json"]).is_err());
    }

    #[test]
    fn format_and_window_parse() {
        let args = Args::try_parse_from([
            "wickra-feature-store",
            "--spec",
            "s.json",
            "--stdin",
            "--format",
            "parquet",
            "--out",
            "m.parquet",
            "--window",
            "500",
        ])
        .unwrap();
        assert_eq!(args.format, Some(Format::Parquet));
        assert_eq!(args.out, Some(PathBuf::from("m.parquet")));
        assert_eq!(args.window, Some(500));
    }
}
