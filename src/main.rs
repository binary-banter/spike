use miette::Diagnostic;
use rust_compiler_construction::elf::ElfFile;
use rust_compiler_construction::parser::{parse_program, PrettyParseError};
use std::fs::{File, Permissions};
use std::io;
use std::io::{Read, stdin};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use thiserror::Error;
use rust_compiler_construction::type_checking::type_check_program;

#[derive(Debug, Error, Diagnostic)]
enum MainError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(#[from] PrettyParseError),
    // #[error(transparent)]
    // #[diagnostic(transparent)]
    // TypeError(#[from] TypeError),
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

    type_check_program(&program).unwrap();

    let program = program
        .uniquify()
        .remove_complex_operands()
        .explicate()
        .select()
        .assign_homes()
        .patch()
        .conclude();

    // dbg!(&program);

    let (entry, program) = program.emit();

    dbg!(entry);
    // dbg!(&program);

    let elf = ElfFile::new(entry, &program);
    let mut file = File::create("output").unwrap();
    elf.write(&mut file);
    drop(file);

    Command::new("chmod").arg("+x").arg("output").spawn().unwrap().wait().unwrap();

    Ok(())
}
