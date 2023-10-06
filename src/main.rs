use rust_compiler_construction::parser::{parse_program, PrettyParseError};
use rust_compiler_construction::type_checking::{type_check_program, TypeError};
use std::fs::File;
use std::io;
use std::io::{stdin, Read, stdout};
use std::process::exit;
use miette::{Diagnostic, GraphicalReportHandler, miette, Report};
use thiserror::Error;
use rust_compiler_construction::language::lvar::LVarProgram;

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
    stdin().read_to_string(&mut program).map_err(MainError::IOResult)?;

    // let program = match parse_program(&program) {
    //     Ok(o) => o,
    //     Err(e) => {
    //         let mut s = io::stdout().lock();
    //         GraphicalReportHandler::new().render_report(&mut s, &e).unwrap();
    //         println!("{}", miette!(e));
    //
    //         // println!("{}",e.diagnostic_source().unwrap());
    //         exit(0);
    //     }
    // };

    let program = parse_program(&program)?;

    // type_check_program(&program)?;

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
