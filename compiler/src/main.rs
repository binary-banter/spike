use clap::Parser;
use compiler::passes::parse::parse::PrettyParseError;
use compiler::passes::validate::error::TypeError;
use compiler::{compile, display, Pass};
use miette::{Diagnostic, IntoDiagnostic};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
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
    /// Specifies the path to an input .jj file. If None, it means stdin is used for input.
    input: Option<String>,

    /// Specifies the path to an output file. If None, it uses the input filename.
    /// If that's also None, it defaults to "output".
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    #[arg(value_enum, short, long, value_name = "PASS")]
    display: Option<Pass>,

    #[arg(short, long)]
    run: bool,
}

fn read_from_stdin() -> Result<String, std::io::Error> {
    let mut program = String::new();
    std::io::stdin().read_to_string(&mut program)?;
    Ok(program)
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    let (program, filename) = match args.input.as_ref() {
        None => (read_from_stdin().into_diagnostic()?, "stdin"),
        Some(file) => (fs::read_to_string(file).into_diagnostic()?, file.as_str()),
    };

    if let Some(pass) = args.display {
        return display(&program, filename, pass);
    }

    let output = args.output.as_deref().unwrap_or_else(|| {
        args.input.as_ref().map_or_else(
            || "output",
            |s| Path::new(s).file_stem().unwrap().to_str().unwrap(),
        )
    });

    compile(&program, filename, Path::new(&output))?;

    if args.run {
        Command::new("chmod")
            .arg("+x")
            .arg(output)
            .output()
            .into_diagnostic()?;

        Command::new(format!("./{output}"))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .into_diagnostic()?;

        fs::remove_file(Path::new(output)).into_diagnostic()?;
    }

    Ok(())
}
