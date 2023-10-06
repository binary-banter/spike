use rust_compiler_construction::parser::parse_program;
use rust_compiler_construction::type_checking::{type_check_program, TypeError};
use std::fs::File;

fn main() -> Result<(), TypeError> {
    let program = parse_program("(print (- (read) (read)))").unwrap().1;

    type_check_program(&program)?;

    let mut output = File::create("output.s").unwrap();

    program
        .uniquify()
        .remove_complex_operands()
        .explicate()
        .select()
        .assign_homes()
        .patch()
        .conclude()
        .emit(&mut output)
        .unwrap();

    Ok(())
}
