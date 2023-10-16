use crate::language::lvar::Def;
use crate::parser::r#fn::parse_fn;
use nom::branch::alt;
use nom::IResult;
use nom::Parser;

pub fn parse_def(input: &str) -> IResult<&str, Def<&str>> {
    alt((parse_fn,)).parse(input)
}
