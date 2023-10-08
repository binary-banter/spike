use miette::Diagnostic;
use rust_compiler_construction::elf::ElfFile;
use rust_compiler_construction::parser::{parse_program, PrettyParseError};
use rust_compiler_construction::type_checking::{type_check_program, TypeError};
use std::fs::File;
use std::io;
use std::io::{stdin, Read};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
enum MainError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(#[from] PrettyParseError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    #[diagnostic()]
    IOResult(#[from] io::Error),
}

fn main() -> miette::Result<()> {
    let mut program = String::new();
    stdin()
        .read_to_string(&mut program)
        .map_err(MainError::IOResult)?;

    let program = parse_program(&program)?;

    type_check_program(&program)?;

    let program = program
        .uniquify()
        .remove_complex_operands()
        .explicate()
        .select()
        .assign_homes()
        .patch()
        .conclude();

    let (entry, program) = program.emit();

    let elf = ElfFile::new(entry, &program);
    let mut file = File::create("output").unwrap();
    elf.write(&mut file);

    Ok(())
}
