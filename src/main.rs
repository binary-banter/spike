use crate::parser::parse_program;
use crate::passes::explicate_control::explicate_program;
use crate::passes::remove_complex_operands::rco_program;
use crate::passes::select_instructions::select_program;
use crate::passes::uniquify::uniquify_program;
use crate::type_checking::{type_check_program, TypeError};

pub mod interpreter;
pub mod language;
mod parser;
pub mod passes;
mod type_checking;
pub mod utils;

fn main() -> Result<(), TypeError> {
    let program = parse_program("(let (x (+ 1 (let (y 1) y))) x)").unwrap().1;

    type_check_program(&program)?;

    dbg!(select_program(explicate_program(rco_program(
        uniquify_program(program)
    ))));

    Ok(())
}
