use rust_compiler_construction::parser::parse_program;
use rust_compiler_construction::type_checking::{type_check_program, TypeError};
use std::fs::File;
use std::io;
use std::io::{stdin, Read};
use thiserror::Error;

#[derive(Debug, Error)]
enum MainError {
    #[error(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    IOResult(#[from] io::Error),
}

fn main() -> Result<(), MainError> {
    let mut program = String::new();
    stdin().read_to_string(&mut program)?;
    let program = parse_program(&program).unwrap().1;

    type_check_program(&program)?;

    program
        .uniquify()
        .remove_complex_operands()
        .explicate()
        .select()
        .assign_homes()
        .patch()
        .conclude()
        .emit(&mut File::create("output.s").unwrap())
        .unwrap();

    Ok(())
}
