//! Reference command-line feature store: fold a `FeatureSpec` over CSV candle
//! files (or a JSON dataset on stdin) and emit the feature matrix as JSON, CSV,
//! Arrow or Parquet.

mod args;
mod run;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = args::Args::parse();
    match run::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
