#![allow(clippy::module_inception)]

pub mod interpreter;
pub mod passes;
pub mod utils;

use crate::passes::parse::parse::parse_program;
use std::fs::File;
use std::path::Path;

pub fn compile(program: &str, output: &Path) -> miette::Result<()> {
    let mut file = File::create(output).unwrap();

    parse_program(program)?
        .type_check()?
        .uniquify()
        .reveal()
        .atomize()
        .explicate()
        .eliminate()
        .select()
        .add_liveness()
        .compute_interference()
        .color_interference()
        .assign_homes()
        .patch()
        .conclude()
        .emit()
        .write(&mut file);

    Ok(())
}
