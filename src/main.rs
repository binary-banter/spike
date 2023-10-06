use crate::parser::parse_program;
use crate::passes::assign_homes::assign_program;
use crate::passes::conclude::conclude_program;
use crate::passes::emit::emit_program;
use crate::passes::explicate_control::explicate_program;
use crate::passes::patch_instructions::patch_program;
use crate::passes::remove_complex_operands::rco_program;
use crate::passes::select_instructions::select_program;
use crate::passes::uniquify::uniquify_program;
use crate::type_checking::{type_check_program, TypeError};
use std::fs::File;

pub mod interpreter;
pub mod language;
mod parser;
pub mod passes;
mod type_checking;
pub mod utils;

fn main() -> Result<(), TypeError> {
    let program = parse_program("(print (+ 10 (read)))").unwrap().1;

    type_check_program(&program)?;

    let program = conclude_program(patch_program(assign_program(select_program(
        explicate_program(rco_program(uniquify_program(program))),
    ))));

    let mut output = File::create("output.s").unwrap();
    emit_program(program, &mut output).unwrap();

    Ok(())
}
