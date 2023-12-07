#![allow(clippy::module_inception)]

#[cfg(feature = "debug")]
pub mod debug;
pub mod passes;
pub mod utils;

use crate::passes::parse::parse::parse;
use miette::{IntoDiagnostic, NamedSource, Report};
use std::fs::File;
use std::path::Path;

pub fn compile(program: &str, filename: &str, output: &Path) -> miette::Result<()> {
    time_init!();

    let add_source =
        |error| Report::with_source_code(error, NamedSource::new(filename, program.to_string()));

    parse(program)
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
