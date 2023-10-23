use crate::language::lvar::Expr;
use crate::parser::expression::parse_expression;

use crate::parser::{trim0, trim1};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::sequence::{delimited, tuple};
use nom::{IResult, Parser};

pub fn parse_if(input: &str) -> IResult<&str, Expr<&str>> {
    delimited(
        char('('),
        tuple((
            trim0(tag("if")),
            trim1(parse_expression),
            trim1(parse_expression),
            trim1(parse_expression),
        )),
        trim0(char(')')),
    )
    .map(|(_, cnd, thn, els)| Expr::If {
        cnd: Box::new(cnd),
        thn: Box::new(thn),
        els: Box::new(els),
    })
    .parse(input)
}
