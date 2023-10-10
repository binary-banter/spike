use crate::language::lvar::Expr;
use nom::branch::alt;
use nom::character::complete::char;
use nom::{IResult, Parser};

pub fn parse_bool(input: &str) -> IResult<&str, Expr<&str>> {
    alt((
        char('f').map(|_| Expr::Bool { val: false }),
        char('t').map(|_| Expr::Bool { val: true }),
    ))(input)
}
