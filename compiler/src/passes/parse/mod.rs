mod grammar;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use crate::language::lvar::PrgParsed;
use crate::passes::parse::grammar::{ProgramParser};


#[derive(Error, Debug, Diagnostic)]
#[error("Parse error!")]
#[diagnostic(
code(oops::my::bad),
url(docsrs),
help("try doing it better next time?")
)]
pub struct PrettyParseError {
    #[source_code]
    src: String,

    #[label("Failed to parse here")]
    fail: SourceSpan,
}

pub fn parse_program(src: &str) -> Result<PrgParsed, PrettyParseError> {
    ProgramParser::new().parse(src).map_err(|e| {
        dbg!(e);
        panic!();
    })
}
