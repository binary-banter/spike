use crate::explicate_control::explicate_program;
use crate::parser::parse_program;
use crate::remove_complex_operands::rco_program;
use crate::select_instructions::select_program;
use crate::uniquify::uniquify_program;

mod alvar;
mod cvar;
mod elvar;
mod explicate_control;
mod lvar;
mod parser;
mod remove_complex_operands;
mod select_instructions;
mod uniquify;
mod x86var;

fn main() {
    dbg!(select_program(explicate_program(rco_program(
        uniquify_program(parse_program("(let (x (+ 1 (let (y 1) y))) x)").unwrap().1)
    ))));
}
