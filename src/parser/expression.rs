use crate::language::lvar::Expr;
use crate::parser::apply::parse_apply;
use crate::parser::bool::parse_bool;
use crate::parser::int::parse_int;
use crate::parser::prim::parse_prim;
use crate::parser::r#if::parse_if;
use crate::parser::r#let::parse_let;
use crate::parser::var::parse_var;
use nom::branch::alt;
use nom::IResult;

pub fn parse_expression(input: &str) -> IResult<&str, Expr<&str>> {
    alt((
        parse_prim,
        parse_apply,
        parse_bool,
        parse_int,
        parse_var,
        parse_let,
        parse_if,
    ))(input)
}

pub fn parse_expression_no_apply(input: &str) -> IResult<&str, Expr<&str>> {
    alt((
        parse_prim,
        parse_bool,
        parse_int,
        parse_var,
        parse_let,
        parse_if,
    ))(input)
}
