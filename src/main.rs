use rust_compiler_construction::parser::parse_program;
use rust_compiler_construction::passes::assign_homes::assign_program;
use rust_compiler_construction::passes::conclude::conclude_program;
use rust_compiler_construction::passes::emit::emit_program;
use rust_compiler_construction::passes::explicate_control::explicate_program;
use rust_compiler_construction::passes::patch_instructions::patch_program;
use rust_compiler_construction::passes::remove_complex_operands::rco_program;
use rust_compiler_construction::passes::select_instructions::select_program;
use rust_compiler_construction::passes::uniquify::uniquify_program;
use rust_compiler_construction::type_checking::{type_check_program, TypeError};
use std::fs::File;

fn main() -> Result<(), TypeError> {
    let program = parse_program("(print (- (read) (read)))").unwrap().1;

    type_check_program(&program)?;

    let program = conclude_program(patch_program(assign_program(select_program(
        explicate_program(rco_program(uniquify_program(program))),
    ))));

    let mut output = File::create("output.s").unwrap();
    emit_program(program, &mut output).unwrap();

    Ok(())
}
