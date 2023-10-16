mod bool;
mod def;
mod expression;
mod r#fn;
mod identifier;
mod r#if;
mod int;
mod r#let;
mod operation;
mod prim;
mod r#type;
mod var;

use crate::language::lvar::LVarProgram;
use crate::parser::def::parse_def;
use crate::parser::expression::parse_expression;
use miette::{Diagnostic, SourceOffset, SourceSpan};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::all_consuming;
use nom::error::{ErrorKind, ParseError};
use nom::multi::many0;
use nom::sequence::{pair, preceded, terminated};
use nom::Err;
use nom::{IResult, Parser, Slice};
use regex::Regex;
use thiserror::Error;

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

pub fn parse_program(src: &str) -> Result<LVarProgram, PrettyParseError> {
    match parse_program_sub(src) {
        Ok((_, o)) => Ok(o),
        Err(e) => {
            let e = match e {
                Err::Incomplete(_) => unreachable!(),
                Err::Error(e) => e,
                Err::Failure(e) => e,
            };

            let offset = src.len() - e.input.len();
            Err(PrettyParseError {
                src: src.to_string(),
                fail: SourceSpan::new(SourceOffset::from(offset), SourceOffset::from(1)),
            })
        }
    }
}

fn parse_program_sub(input: &str) -> IResult<&str, LVarProgram> {
    all_consuming(pair(
        many0(parse_def),
        terminated(trim0(parse_expression), multispace0),
    ))
    .map(|(defs, bdy)| LVarProgram { defs, bdy })
    .parse(input)
}

fn trim0<'a, O, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, O, E>,
) -> impl Parser<&'a str, O, E> {
    preceded(multispace0, parser)
}

fn trim1<'a, O, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, O, E>,
) -> impl Parser<&'a str, O, E> {
    preceded(multispace1, parser)
}

fn re_find<'a, 'b, E>(re: &'b Regex) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, E> + 'b
where
    E: ParseError<&'a str>,
{
    move |i| {
        if let Some(m) = re.find(i) {
            Ok((i.slice(m.end()..), i.slice(m.start()..m.end())))
        } else {
            Err(Err::Error(E::from_error_kind(i, ErrorKind::RegexpFind)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn parse([test]: [&str; 1]) {
        split_test(test);
    }

    test_each_file! { for ["test"] in "./programs/good" as parse => parse }
}
