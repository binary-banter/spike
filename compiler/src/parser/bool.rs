use crate::language::lvar::{Expr, Lit};
use nom::branch::alt;
use nom::character::complete::char;
use nom::{IResult, Parser};

pub fn parse_bool(input: &str) -> IResult<&str, Expr<&str>> {
    alt((char('f').map(|_| false), char('t').map(|_| true)))
        .map(|val| Expr::Lit {
            val: Lit::Bool { val },
        })
        .parse(input)
}