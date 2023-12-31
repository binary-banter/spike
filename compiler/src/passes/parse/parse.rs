use crate::passes::parse::grammar::ProgramParser;
use crate::passes::parse::PrgParsed;
#[cfg(test)]
use derive_name::VariantName;
use itertools::Itertools;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;
use miette::Diagnostic;
use thiserror::Error;

#[cfg_attr(test, derive(VariantName))]
#[derive(Error, Debug, Diagnostic)]
pub enum PrettyParseError {
    #[error("Parse error: Invalid token.")]
    InvalidToken {
        #[label("Failed to parse here, invalid token starting here")]
        fail: (usize, usize),
    },
    #[error("Parse error: Unexpected token.")]
    UnexpectedToken {
        #[label("Failed to parse here, expected one of: {expected}")]
        fail: (usize, usize),
        expected: String,
    },
    #[error("Parse error: Unexpected end of file.")]
    UnexpectedEOF {
        #[label("Unexpected end-of-file, expected one of: {expected}")]
        fail: (usize, usize),
        expected: String,
    },
}

pub fn parse_program(src: &str) -> Result<PrgParsed, PrettyParseError> {
    ProgramParser::new().parse(src).map_err(From::from)
}

impl<'p> From<ParseError<usize, Token<'p>, &'p str>> for PrettyParseError {
    fn from(value: ParseError<usize, Token<'p>, &'p str>) -> Self {
        match value {
            ParseError::InvalidToken { location } => PrettyParseError::InvalidToken {
                fail: (location, 1),
            },
            ParseError::UnrecognizedEof { location, expected } => PrettyParseError::UnexpectedEOF {
                fail: (location, 1),
                expected: expected.into_iter().format(", ").to_string(),
            },
            ParseError::UnrecognizedToken { token, expected } => {
                PrettyParseError::UnexpectedToken {
                    fail: (token.0, token.2 - token.0),
                    expected: expected.into_iter().format(", ").to_string(),
                }
            }
            ParseError::ExtraToken { .. } => {
                unreachable!("Our grammar always consumes the entire input.")
            }
            ParseError::User { .. } => unreachable!("No custom `ParseError`s are implemented."),
        }
    }
}
