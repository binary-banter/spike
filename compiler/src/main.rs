use crate::MainError::{IOResult, RegexError};
use clap::Parser;
use compiler::compile;
use compiler::parser::PrettyParseError;
use compiler::passes::type_check::TypeError;
use miette::Diagnostic;
use regex::Regex;
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
    #[error(transparent)]
    #[diagnostic()]
    RegexError(#[from] regex::Error),
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

    let file_regex = Regex::new(r"(?<name>[^\\/]+)(?:\.jj)$").map_err(RegexError)?;

    let output = args.output.unwrap_or_else(|| {
        args.input
            .as_ref()
            .and_then(|s| {
                file_regex
                    .captures(s)
                    .and_then(|c| c.name("name"))
                    .map(|m| m.as_str())
            })
            .map_or_else(|| "output".to_string(), str::to_string)
    });

    compile(&program, Path::new(&output))
}
