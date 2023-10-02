mod expression;
mod identifier;
mod int;
mod r#let;
mod operation;
mod prim;
mod var;

use crate::LVarProgram;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::all_consuming;
use nom::error::{ErrorKind, ParseError};
use nom::sequence::{preceded, terminated};
use nom::Err;
use nom::{IResult, Parser, Slice};
use regex::Regex;

#[allow(unused)]
pub enum Type {
    Integer,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Read,
    Print,
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Var {
        sym: String,
    },
    Prim {
        op: Operation,
        args: Vec<Expr>,
    },
    Let {
        sym: String,
        bnd: Box<Expr>,
        bdy: Box<Expr>,
    },
}

pub fn parse_program(input: &str) -> IResult<&str, LVarProgram> {
    all_consuming(terminated(expression::parse_expression, multispace0))
        .map(|body| LVarProgram { bdy: body })
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
    use crate::parser::{parse_program, Expr};
    use crate::LVarProgram;

    #[test]
    fn int() {
        assert_eq!(
            parse_program("42").unwrap().1,
            LVarProgram { bdy: Expr::Int(42) }
        )
    }
}
