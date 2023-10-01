use crate::parser::{parse_program, Expression};
use crate::uniquify::uniquify_program;

mod parser;
mod uniquify;

fn main() {
    let mut program = parse_program("(let (x 42) x)").unwrap().1;
    uniquify_program(&mut program);
}

#[derive(Debug, PartialEq)]
pub struct LVarProgram {
    pub body: Expression,
}
