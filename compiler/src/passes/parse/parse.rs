use crate::passes::parse::grammar::ProgramParser;
use crate::passes::parse::PrgParsed;
use itertools::Itertools;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;
use miette::{Diagnostic, NamedSource};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum PrettyParseError {
    #[error("Parse error: Invalid token.")]
    InvalidToken {
        #[source_code]
        src: NamedSource,
        #[label("Failed to parse here, invalid token starting here.")]
        fail: (usize, usize),
    },
    #[error("Parse error: Unexpected token.")]
    UnexpectedToken {
        #[source_code]
        src: NamedSource,
        #[label("Failed to parse here, expected one of: {expected}.")]
        fail: (usize, usize),
        expected: String,
    },
    #[error("Parse error: Unexpected end of file.")]
    UnexpectedEOF {
        #[source_code]
        src: NamedSource,
        #[label("Unexpected end-of-file, expected one of: {expected}.")]
        fail: (usize, usize),
        expected: String,
    },
}

pub fn parse_program<'p>(src: &'p str, file: &'p str) -> Result<PrgParsed<'p>, PrettyParseError> {
    ProgramParser::new()
        .parse(src)
        .map_err(|error| prettify_error(error, src, file))
}

fn prettify_error<'p>(
    err: ParseError<usize, Token<'p>, &'p str>,
    src: &'p str,
    file: &'p str,
) -> PrettyParseError {
    let src = NamedSource::new(file, src.to_string());

    match err {
        ParseError::InvalidToken { location } => PrettyParseError::InvalidToken {
            src,
            fail: (location, 1),
        },
        ParseError::UnrecognizedEof { location, expected } => PrettyParseError::UnexpectedEOF {
            src,
            fail: (location, 1),
            expected: expected.into_iter().format(", ").to_string(),
        },
        ParseError::UnrecognizedToken { token, expected } => PrettyParseError::UnexpectedToken {
            src,
            fail: (token.0, token.2 - token.0),
            expected: expected.into_iter().format(", ").to_string(),
        },
        ParseError::ExtraToken { .. } => unreachable!("Our grammar always consumes the entire input."),
        ParseError::User { .. } => unreachable!("No custom `ParseError`s are implemented."),
    }
}
