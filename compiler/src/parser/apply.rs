use crate::language::lvar::Expr;
use crate::parser::expression::{parse_expression, parse_expression_no_apply};
use crate::parser::trim0;
use nom::character::complete::char;
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair};
use nom::IResult;
use nom::Parser;

pub fn parse_apply(input: &str) -> IResult<&str, Expr<&str>> {
    pair(
        parse_expression_no_apply,
        delimited(
            trim0(char('(')),
            separated_list0(trim0(char(',')), trim0(parse_expression)),
            trim0(char(')')),
        ),
    )
    .map(|(fun, args)| Expr::Apply {
        fun: Box::new(fun),
        args,
    })
    .parse(input)
}
