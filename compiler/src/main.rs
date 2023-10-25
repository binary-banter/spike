use crate::MainError::IOResult;
use clap::Parser;
use compiler::compile;
use compiler::passes::parse::parse::PrettyParseError;
use compiler::passes::type_check::check::TypeError;
use miette::Diagnostic;
use std::io::Read;
use std::path::Path;
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
enum MainError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(#[from] PrettyParseError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    #[diagnostic()]
    IOResult(#[from] io::Error),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specifies the path to an input .jj file. If None, it means stdin is used for input.
    input: Option<String>,

    /// Specifies the path to an output file. If None, it uses the input filename.
    /// If that's also None, it defaults to "output".
    #[arg(short, long)]
    output: Option<String>,
}

fn read_from_stdin() -> Result<String, io::Error> {
    let mut program = String::new();
    io::stdin().read_to_string(&mut program)?;
    Ok(program)
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    let program = args
        .input
        .as_ref()
        .map_or_else(read_from_stdin, fs::read_to_string)
        .map_err(IOResult)?;

    let output: &str = args.output.as_deref().unwrap_or_else(|| {
        args.input.as_ref().map_or_else(
            || "output",
            |s| Path::new(s).file_stem().unwrap().to_str().unwrap(),
        )
    });

    compile(&program, Path::new(&output))
}
