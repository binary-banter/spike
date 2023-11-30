#![allow(clippy::module_inception)]

pub mod interpreter;
pub mod passes;
pub mod utils;

use crate::passes::parse::parse::parse_program;
use clap::ValueEnum;
use miette::{IntoDiagnostic, NamedSource, Report};
use std::fs::File;
use std::path::Path;

#[derive(ValueEnum, Clone, Debug)]
pub enum Pass {
    Parse,
    Validate,
    Reveal,
    Atomize,
}

pub fn compile(program: &str, filename: &str, output: &Path) -> miette::Result<()> {
    let add_source =
        |error| Report::with_source_code(error, NamedSource::new(filename, program.to_string()));

    parse_program(program)
        .map_err(Into::into)
        .map_err(add_source)?
        .validate()
        .map_err(Into::into)
        .map_err(add_source)?
        .reveal()
        .atomize()
        .explicate()
        .eliminate()
        .select()
        .assign()
        .patch()
        .conclude()
        .emit()
        .write(&mut File::create(output).into_diagnostic()?);

    Ok(())
}

pub fn display(program: &str, filename: &str, pass: Pass) -> miette::Result<()> {
    let add_source =
        |error| Report::with_source_code(error, NamedSource::new(filename, program.to_string()));

    let prg_parsed = parse_program(program)
        .map_err(Into::into)
        .map_err(add_source)?;

    display!(prg_parsed, pass, Parse);

    let prg_validated = prg_parsed
        .validate()
        .map_err(Into::into)
        .map_err(add_source)?;

    display!(prg_validated, pass, Validate);

    let prg_revealed = prg_validated.reveal();

    display!(prg_revealed, pass, Reveal);

    let prg_atomized = prg_revealed.atomize();

    display!(prg_atomized, pass, Atomize);

    Ok(())
}

#[macro_export]
macro_rules! display {
    ($prg:expr, $pass:expr, $pat:ident) => {
        if matches!($pass, $crate::Pass::$pat) {
            println!("{}", $prg);
            return Ok(());
        }
    };
}
