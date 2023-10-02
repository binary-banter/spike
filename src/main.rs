use crate::parser::{parse_program, Expr};
use crate::remove_complex_operands::rco_program;
use crate::uniquify::uniquify_program;

mod parser;
mod remove_complex_operands;
mod uniquify;

fn main() {
    let _program = rco_program(uniquify_program(parse_program("(let (x 42) x)").unwrap().1));
}

#[derive(Debug, PartialEq)]
pub struct LVarProgram {
    pub bdy: Expr,
}
