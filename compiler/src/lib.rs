#![allow(clippy::module_inception)]

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
    Explicate,
    Select,
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

    let prg_parsed = display!(
        parse_program(program)
            .map_err(Into::into)
            .map_err(add_source)?,
        pass,
        Parse
    );
    let prg_validated = display!(
        prg_parsed
            .validate()
            .map_err(Into::into)
            .map_err(add_source)?,
        pass,
        Validate
    );
    let prg_revealed = display!(prg_validated.reveal(), pass, Reveal);
    let prg_atomized = display!(prg_revealed.atomize(), pass, Atomize);
    let prg_explicated = display!(prg_atomized.explicate(), pass, Explicate);
    let _prg_select = display!(prg_explicated.eliminate().select(), pass, Select);

    Ok(())
}

#[macro_export]
macro_rules! display {
    ($prg:expr, $pass:expr, $pat:ident) => {
        if matches!($pass, $crate::Pass::$pat) {
            println!("{}", $prg);
            return Ok(());
        } else {
            $prg
        }
    };
}
