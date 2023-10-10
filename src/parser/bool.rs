use crate::language::lvar::Expr;
use nom::branch::alt;
use nom::character::complete::char;
use nom::{IResult, Parser};
use crate::interpreter::value::Val;

pub fn parse_bool(input: &str) -> IResult<&str, Expr<&str>> {
    alt((
        char('f').map(|_| false),
        char('t').map(|_| true),
    )).map(|val| Expr::Val { val: Val::Bool { val } }).parse(input)
}
