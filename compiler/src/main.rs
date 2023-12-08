#[cfg(unix)]
mod run;

use clap::Parser;
use compiler::compile;
#[cfg(feature = "debug")]
use compiler::debug::{DebugArgs, Pass};
use compiler::passes::{parse::parse::PrettyParseError, validate::error::TypeError};
use miette::{Diagnostic, IntoDiagnostic};
use std::fs;
use std::io::{read_to_string, stdin};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
enum MainError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(#[from] PrettyParseError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    ValidateError(#[from] TypeError),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specifies a path to an input .sp file. If None, it means stdin is used for input.
    input: Option<String>,

    /// Specifies a path to an output file. If None, it uses the input filename.
    /// If that's also None, it defaults to "output".
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Optionally runs and deletes the compiled executable. Only supported on Unix systems.
    #[cfg(unix)]
    #[arg(short, long)]
    run: bool,

    /// Specifies a pass to display.
    #[cfg(feature = "debug")]
    #[arg(value_enum, short, long, value_name = "PASS")]
    display: Option<Pass>,

    /// Print timing debug information.
    #[cfg(feature = "debug")]
    #[arg(short, long)]
    time: bool,
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    #[cfg(feature = "debug")]
    DebugArgs::set(args.time, args.display)?;

    let (program, filename) = match args.input.as_ref() {
        None => (read_to_string(stdin()).into_diagnostic()?, "stdin"),
        Some(file) => (fs::read_to_string(file).into_diagnostic()?, file.as_str()),
    };

    let output = args.output.as_deref().unwrap_or_else(|| {
        args.input.as_ref().map_or_else(
            || "output",
            |s| Path::new(s).file_stem().unwrap().to_str().unwrap(),
        )
    });

    compile(&program, filename, Path::new(&output))?;

    #[cfg(unix)]
    if args.run {
        run::run(output)?;
    }

    Ok(())
}
