use miette::{Diagnostic};
use rust_compiler_construction::parser::{parse_program, PrettyParseError};
use std::fs::{File, Permissions};
use std::io;
use std::io::{stdin, Read};
use thiserror::Error;
use rust_compiler_construction::elf::ElfFile;

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
    let elf = ElfFile::new();
    let mut file = File::create("output").unwrap();
    elf.emit(&mut file);



    // let mut program = String::new();
    // stdin()
    //     .read_to_string(&mut program)
    //     .map_err(MainError::IOResult)?;
    //
    // // let program = match parse_program(&program) {
    // //     Ok(o) => o,
    // //     Err(e) => {
    // //         let mut s = io::stdout().lock();
    // //         GraphicalReportHandler::new().render_report(&mut s, &e).unwrap();
    // //         println!("{}", miette!(e));
    // //
    // //         // println!("{}",e.diagnostic_source().unwrap());
    // //         exit(0);
    // //     }
    // // };
    //
    // let program = parse_program(&program)?;
    //
    // // type_check_program(&program)?;
    //
    // program
    //     .uniquify()
    //     .remove_complex_operands()
    //     .explicate()
    //     .select()
    //     .assign_homes()
    //     .patch()
    //     .conclude()
    //     .emit(&mut File::create("output.s").unwrap())
    //     .unwrap();
    //
    Ok(())
}
